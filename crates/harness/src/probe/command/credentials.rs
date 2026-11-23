use super::super::ProbeError;
use aws_wire::sign::Credentials;
use inventory_sweep::{CredentialFailure, CredentialSource};

pub(super) struct EnvironmentCredentials {
    region: String,
    credentials: Credentials,
}

impl EnvironmentCredentials {
    pub(super) fn load(region: String) -> Result<Self, ProbeError> {
        let key = std::env::var("AWS_ACCESS_KEY_ID")
            .map_err(|_| ProbeError::Invalid("probe requires AWS_ACCESS_KEY_ID"))?;
        let secret = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| ProbeError::Invalid("probe requires AWS_SECRET_ACCESS_KEY"))?;
        let token = match std::env::var("AWS_SESSION_TOKEN") {
            Ok(token) => Some(token),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => return Err(ProbeError::Invalid("AWS_SESSION_TOKEN must be UTF-8")),
        };
        Ok(Self {
            region,
            credentials: Credentials::new(&key, &secret, token.as_deref())?,
        })
    }
}

impl CredentialSource for EnvironmentCredentials {
    fn region(&self) -> &str {
        &self.region
    }

    async fn resolve(&self) -> Result<Credentials, CredentialFailure> {
        Ok(self.credentials.clone())
    }
}
