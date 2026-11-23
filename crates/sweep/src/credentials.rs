use aws_wire::sign::Credentials;

pub trait CredentialSource: Send + Sync {
    fn region(&self) -> &str;
    fn resolve(&self) -> impl Future<Output = Result<Credentials, CredentialFailure>> + Send;
}

#[derive(Debug)]
pub struct CredentialFailure {
    pub retryable: bool,
    pub message: String,
}

#[cfg(test)]
pub(crate) struct TestCredentials(String);

#[cfg(test)]
impl TestCredentials {
    pub(crate) fn new(region: &str) -> Self {
        Self(region.to_owned())
    }
}

#[cfg(test)]
impl CredentialSource for TestCredentials {
    fn region(&self) -> &str {
        &self.0
    }

    fn resolve(&self) -> impl Future<Output = Result<Credentials, CredentialFailure>> + Send {
        std::future::ready(
            Credentials::new("fixture-key", "fixture-secret", None).map_err(|error| {
                CredentialFailure {
                    retryable: false,
                    message: error.to_string(),
                }
            }),
        )
    }
}
