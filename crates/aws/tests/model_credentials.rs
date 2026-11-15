use inventory_aws::model_credentials::{CredentialError, ProfileCredentials};

#[tokio::test]
async fn explicit_region_is_used_without_resolving_credentials() {
    let profile = ProfileCredentials::load(Some("isolated-profile"), Some("us-east-2")).await;
    assert!(profile.is_ok());
    let profile = profile.map_err(|error| error.to_string());
    assert_eq!(
        profile.as_ref().map(ProfileCredentials::region),
        Ok("us-east-2")
    );
}

#[tokio::test]
async fn invalid_profile_fails_before_ambient_configuration_is_loaded() {
    let result = ProfileCredentials::load(Some("\ninvalid"), Some("us-east-2")).await;
    assert!(matches!(result, Err(CredentialError::InvalidProfile)));
}

#[tokio::test]
async fn invalid_region_fails_before_ambient_configuration_is_loaded() {
    let result = ProfileCredentials::load(Some("isolated-profile"), Some("\ninvalid")).await;
    assert!(matches!(result, Err(CredentialError::InvalidRegion)));
}
