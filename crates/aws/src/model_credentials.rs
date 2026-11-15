use std::error::Error;
use std::fmt;

use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
use aws_types::region::Region;

const MAX_CONFIG_VALUE_BYTES: usize = 256;

#[derive(Clone)]
pub struct ProfileCredentials {
    provider: SharedCredentialsProvider,
    region: String,
}

impl fmt::Debug for ProfileCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileCredentials")
            .field("region", &self.region)
            .field("credentials", &"[REDACTED]")
            .finish()
    }
}

#[non_exhaustive]
pub enum CredentialError {
    InvalidProfile,
    InvalidRegion,
    UnresolvedRegion,
    MissingProvider,
    Provider(CredentialsError),
    Wire(aws_wire::sign::Error),
}

impl fmt::Debug for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::InvalidProfile => "InvalidProfile",
            Self::InvalidRegion => "InvalidRegion",
            Self::UnresolvedRegion => "UnresolvedRegion",
            Self::MissingProvider => "MissingProvider",
            Self::Provider(_) => "Provider",
            Self::Wire(_) => "Wire",
        };
        formatter
            .debug_tuple("CredentialError")
            .field(&name)
            .finish()
    }
}

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidProfile => "AWS profile name is invalid or too long",
            Self::InvalidRegion => "AWS region is invalid or too long",
            Self::UnresolvedRegion => "AWS region could not be resolved",
            Self::MissingProvider => "AWS credential provider is unavailable",
            Self::Provider(_) => "AWS credential provider failed",
            Self::Wire(_) => "AWS credentials could not be adapted for signing",
        };
        formatter.write_str(message)
    }
}

impl Error for CredentialError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Provider(error) => Some(error),
            Self::Wire(error) => Some(error),
            _ => None,
        }
    }
}

impl ProfileCredentials {
    pub async fn load(
        profile: Option<&str>,
        region: Option<&str>,
    ) -> Result<Self, CredentialError> {
        validate_options(profile, region)?;
        Self::from_config(load_config(profile, region).await)
    }

    fn from_config(config: aws_types::SdkConfig) -> Result<Self, CredentialError> {
        let region = config
            .region()
            .map(|value| value.as_ref().to_owned())
            .ok_or(CredentialError::UnresolvedRegion)?;
        let provider = config
            .credentials_provider()
            .ok_or(CredentialError::MissingProvider)?;
        Ok(Self { provider, region })
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub async fn resolve(&self) -> Result<aws_wire::sign::Credentials, CredentialError> {
        let credentials = self
            .provider
            .provide_credentials()
            .await
            .map_err(CredentialError::Provider)?;
        aws_wire::sign::Credentials::new(
            credentials.access_key_id(),
            credentials.secret_access_key(),
            credentials.session_token(),
        )
        .map_err(CredentialError::Wire)
    }
}

async fn load_config(profile: Option<&str>, region: Option<&str>) -> aws_types::SdkConfig {
    let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
    if let Some(name) = profile {
        loader = loader.profile_name(name);
    }
    if let Some(name) = region {
        loader = loader.region(Region::new(name.to_owned()));
    }
    loader.load().await
}

fn validate_options(profile: Option<&str>, region: Option<&str>) -> Result<(), CredentialError> {
    profile.map(validate_profile).transpose()?;
    region.map(validate_region).transpose()?;
    Ok(())
}

fn validate_profile(profile: &str) -> Result<(), CredentialError> {
    if invalid_config_value(profile) {
        return Err(CredentialError::InvalidProfile);
    }
    Ok(())
}

fn validate_region(region: &str) -> Result<(), CredentialError> {
    if invalid_config_value(region) {
        return Err(CredentialError::InvalidRegion);
    }
    Ok(())
}

fn invalid_config_value(value: &str) -> bool {
    value.is_empty()
        || value.len() > MAX_CONFIG_VALUE_BYTES
        || value.bytes().any(|byte| byte.is_ascii_control())
}

#[cfg(test)]
mod tests {
    use super::{CredentialError, ProfileCredentials};
    use aws_credential_types::Credentials;
    use aws_credential_types::provider::SharedCredentialsProvider;

    #[tokio::test]
    async fn explicit_region_and_safe_provider_resolve_credentials() {
        let provider = SharedCredentialsProvider::new(Credentials::new(
            "TESTACCESS",
            "test-secret-value",
            Some("test-session-token".to_owned()),
            None,
            "isolated-test-provider",
        ));
        let profile = ProfileCredentials {
            provider,
            region: "us-east-2".to_owned(),
        };
        let credentials = profile.resolve().await;
        assert!(credentials.is_ok());
        assert_eq!(profile.region(), "us-east-2");
        let debug = format!("{profile:?}");
        assert!(!debug.contains("test-secret-value"));
        assert!(!debug.contains("test-session-token"));
    }

    #[test]
    fn credential_errors_do_not_display_provider_material() {
        let error = CredentialError::MissingProvider;
        let display = error.to_string();
        let debug = format!("{error:?}");
        assert!(!display.contains("secret"));
        assert!(!debug.contains("secret"));
    }
    #[tokio::test]
    async fn wire_conversion_error_preserves_its_source_without_echoing_values() {
        let provider = SharedCredentialsProvider::new(Credentials::new(
            "",
            "sensitive-but-invalid",
            None,
            None,
            "isolated-invalid-provider",
        ));
        let profile = ProfileCredentials {
            provider,
            region: "us-east-2".to_owned(),
        };
        let error = profile.resolve().await;
        assert!(matches!(error, Err(CredentialError::Wire(_))));
        let error = error.err();
        let source_preserved = match &error {
            Some(source) => std::error::Error::source(source).is_some(),
            None => false,
        };
        assert!(source_preserved);
        let rendered = format!("{error:?}");
        assert!(!rendered.contains("sensitive-but-invalid"));
    }
}
