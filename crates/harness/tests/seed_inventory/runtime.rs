use super::{ACCOUNT, REGION, TestResult};
use aws_wire::{
    sign::{
        Body, Credentials, PathMode, Region as SigningRegion, Request as SignRequest, Scope,
        SigningName,
    },
    transport::{Client, Config as TransportConfig},
};
use bytes::Bytes;
use http::{Method, Request, header::CONTENT_TYPE};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_harness::{
    emulator::Emulator,
    inventory_seed::{ScenarioSeedOracle, SeedConfig, SeedScenario, run_data_scenario},
};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::sync::Semaphore;

pub(super) static NATIVE_SLOT: Semaphore = Semaphore::const_new(1);

pub(super) struct NativeRuntime {
    work: tempfile::TempDir,
    emulator: Emulator,
    catalog: Vec<u8>,
    credentials: Credentials,
}

impl NativeRuntime {
    pub(super) fn start() -> TestResult<Self> {
        let binary = match std::env::var_os("INVENTORY_EMULATOR_BINARY") {
            Some(path) => PathBuf::from(path),
            None => workspace().join(".local/runtime/unpacked/opt/aws-emulator/bin/aws-emulator"),
        };
        let work = tempfile::tempdir()?;
        let emulator = Emulator::start(&binary, work.path(), 300)?;
        let credentials = credentials(emulator.credentials())?;
        let catalog = std::fs::read(catalog_path())?;
        RuntimeCatalog::from_bytes(&catalog)?;
        Ok(Self {
            work,
            emulator,
            catalog,
            credentials,
        })
    }

    pub(super) async fn assert_organization_features(&self) -> TestResult {
        let endpoint = self.emulator.endpoint();
        let mut request = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .header(
                "x-amz-target",
                "AWSOrganizationsV20161128.DescribeOrganization",
            )
            .header(CONTENT_TYPE, "application/x-amz-json-1.1")
            .body(Bytes::from_static(b"{}"))?;
        let scope = Scope {
            region: SigningRegion::new(REGION)?,
            name: SigningName::new("organizations")?,
        };
        let signed = aws_wire::sign::sign(
            &self.credentials,
            &scope,
            SystemTime::now(),
            SignRequest {
                method: request.method(),
                uri: endpoint,
                headers: request.headers(),
                body: Body::Bytes(request.body()),
                payload_header: aws_wire::sign::PayloadHeader::Include,
            },
            PathMode::Standard,
        )?;
        request.headers_mut().extend(signed);
        let response = Client::new(TransportConfig {
            extra_ca_bundle: Some(self.emulator.ca_bundle().to_path_buf()),
            ..TransportConfig::default()
        })?
        .send(request)
        .await?;
        if !response.status.is_success() {
            return Err(format!("DescribeOrganization returned {}", response.status).into());
        }
        let metadata: serde_json::Value = serde_json::from_slice(&response.body)?;
        let feature_set = metadata
            .get("Organization")
            .and_then(|organization| organization.get("FeatureSet"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(feature_set, Some("ALL"));
        Ok(())
    }

    pub(super) async fn seed(&self, scenario: SeedScenario) -> TestResult<ScenarioSeedOracle> {
        if scenario
            .steps()
            .iter()
            .any(|step| step.service == "organizations" && step.operation == "DescribeOrganization")
        {
            self.assert_organization_features().await?;
        }
        let catalog = RuntimeCatalog::from_bytes(&self.catalog)?;
        let config = SeedConfig::new(self.emulator.endpoint(), self.emulator.ca_bundle(), REGION)?;
        let oracle = run_data_scenario(&catalog, config, &self.credentials, scenario).await?;
        assert_eq!(oracle.account_id.as_str(), ACCOUNT);
        assert_eq!(oracle.region.as_str(), REGION);
        Ok(oracle)
    }

    pub(super) fn work(&self) -> &Path {
        self.work.path()
    }
    pub(super) fn emulator(&self) -> &Emulator {
        &self.emulator
    }
    pub(super) fn stop(self) -> TestResult {
        self.emulator.stop()?;
        Ok(())
    }
}

pub(super) fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn catalog_path() -> PathBuf {
    workspace().join("catalog/aws-api.catalog")
}

fn credentials(path: &Path) -> TestResult<Credentials> {
    let text = std::fs::read_to_string(path)?;
    let (id, secret, token) = text.lines().filter_map(|line| line.split_once('=')).fold(
        (None, None, None),
        |(id, secret, token), (key, value)| {
            let value = Some(value.trim());
            match key.trim() {
                "aws_access_key_id" => (value, secret, token),
                "aws_secret_access_key" => (id, value, token),
                "aws_session_token" => (id, secret, value),
                _ => (id, secret, token),
            }
        },
    );
    Ok(Credentials::new(
        id.ok_or("owned emulator credentials lack access key")?,
        secret.ok_or("owned emulator credentials lack secret key")?,
        token,
    )?)
}
