use super::*;
use aws_credential_types::Credentials;
use inventory_sweep::CredentialSource;

type TestResult = Result<(), Box<dyn Error>>;

fn profile(access_key: &str) -> ProfileCredentials {
    ProfileCredentials {
        provider: SharedCredentialsProvider::new(Credentials::new(
            access_key,
            "private-test-secret",
            None,
            None,
            "sweep-contract",
        )),
        region: "us-east-2".to_owned(),
    }
}

#[tokio::test]
async fn given_profile_when_library_resolves_then_region_and_signing_credentials_are_available()
-> TestResult {
    let profile = profile("TESTACCESS");
    assert_eq!(CredentialSource::region(&profile), "us-east-2");
    let credentials = CredentialSource::resolve(&profile).await;
    let credentials = credentials.map_err(|failure| failure.message)?;
    assert_eq!(format!("{credentials:?}"), "Credentials([REDACTED])");
    Ok(())
}

#[tokio::test]
async fn given_invalid_wire_credentials_when_library_resolves_then_failure_is_permanent_and_redacted()
-> TestResult {
    let profile = profile("");
    let result = CredentialSource::resolve(&profile).await;
    let failure = result.err().ok_or("invalid credentials were accepted")?;
    assert!(!failure.retryable);
    assert_eq!(
        failure.message,
        "AWS credentials could not be adapted for signing"
    );
    assert!(!format!("{failure:?}").contains("private-test-secret"));
    Ok(())
}

#[test]
fn given_missing_provider_when_adapted_then_failure_is_permanent() {
    let failure = credential_failure(CredentialError::MissingProvider);
    assert!(!failure.retryable);
    assert_eq!(failure.message, "AWS credential provider is unavailable");
}

#[test]
fn given_provider_error_when_adapted_then_failure_is_retryable_and_redacted() {
    let error =
        CredentialsError::provider_error(std::io::Error::other("private provider material"));
    let failure = credential_failure(CredentialError::Provider(error));
    assert!(failure.retryable);
    assert_eq!(failure.message, "AWS credential provider failed");
    assert!(!format!("{failure:?}").contains("private provider material"));
}
