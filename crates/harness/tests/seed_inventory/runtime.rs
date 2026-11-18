use super::{ACCOUNT, REGION, TestResult};
use aws_wire::sign::Credentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_harness::{
    emulator::Emulator,
    inventory_seed::{ScenarioSeedOracle, SeedConfig, SeedScenario, run_data_scenario},
};
use std::path::{Path, PathBuf};
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

    pub(super) async fn seed(&self, scenario: SeedScenario) -> TestResult<ScenarioSeedOracle> {
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
