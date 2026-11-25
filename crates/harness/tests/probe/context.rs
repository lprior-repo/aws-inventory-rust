use super::server::{Server, TestResult};
use aws_wire::{
    sign::Credentials,
    transport::{Client, Config},
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, RegionId},
};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use inventory_sweep::{CredentialFailure, CredentialSource, ProbeAnswer, SweepError};

pub struct Context {
    client: Client,
    executor: ApiExecutor,
    credentials: TestCredentials,
    region: RegionId,
    endpoint: String,
}

impl Context {
    pub fn new(server: &Server) -> TestResult<Self> {
        Ok(Self {
            client: Client::new(Config {
                extra_ca_bundle: Some(server.ca.clone()),
                ..Config::default()
            })?,
            executor: ApiExecutor::new(ExecutorConfig {
                max_attempts: 1,
                ..ExecutorConfig::default()
            })?,
            credentials: TestCredentials,
            region: RegionId::parse("us-east-1")?,
            endpoint: server.endpoint()?,
        })
    }

    pub async fn report(&self, catalog: &RuntimeCatalog<'_>) -> TestResult<String> {
        Ok(inventory_harness::probe::run(
            catalog,
            &self.credentials,
            &self.client,
            &self.executor,
            &self.region,
            &self.endpoint,
            &[],
        )
        .await?)
    }

    pub async fn once(
        &self,
        catalog: &RuntimeCatalog<'_>,
        operation: &str,
        account: Option<&AccountId>,
    ) -> Result<ProbeAnswer, SweepError> {
        let plan = catalog
            .lookup("bindings", operation)
            .ok_or(SweepError::InvalidConfig("fixture plan missing"))?;
        inventory_sweep::call_once(
            catalog,
            &self.credentials,
            &self.client,
            &self.executor,
            plan,
            inventory_sweep::ProbeScope {
                region: &self.region,
                account,
                endpoint_override: Some(&self.endpoint),
            },
        )
        .await
    }
}

struct TestCredentials;

impl CredentialSource for TestCredentials {
    fn region(&self) -> &str {
        "us-east-1"
    }

    async fn resolve(&self) -> Result<Credentials, CredentialFailure> {
        Credentials::new("TEST", "TEST", None).map_err(|error| CredentialFailure {
            retryable: false,
            message: error.to_string(),
        })
    }
}
