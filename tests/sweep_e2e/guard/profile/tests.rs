use super::validate;

const CREDENTIALS: &str =
    "[default]\naws_access_key_id=fixture-key\naws_secret_access_key=fixture-secret\n";
const ENDPOINT: &str = "https://127.0.0.1:47666";

#[test]
fn static_credentials_accept_normalized_profile_headers() -> anyhow::Result<()> {
    for heading in [
        "[default]",
        "[ default ]",
        "[profile default]",
        "[profile   default]",
    ] {
        assert!(validate(
            CREDENTIALS,
            &format!("{heading}\nregion=us-east-1\nendpoint_url={ENDPOINT}\n"),
            "default",
            ENDPOINT
        )?);
    }
    Ok(())
}

#[test]
fn provider_chains_cannot_hide_in_either_selected_profile_file() -> anyhow::Result<()> {
    for key in [
        "role_arn",
        "source_profile",
        "credential_process",
        "credential_source",
        "sso_session",
        "sso_start_url",
        "web_identity_token_file",
    ] {
        let property = format!("{key}=forbidden\n");
        assert!(!validate(
            &format!("{CREDENTIALS}{property}"),
            "",
            "default",
            ENDPOINT
        )?);
        assert!(!validate(
            CREDENTIALS,
            &format!("[profile   default]\n{property}"),
            "default",
            ENDPOINT
        )?);
    }
    Ok(())
}

#[test]
fn ambiguous_or_malformed_profile_properties_fail_closed() {
    for config in [
        "[default]\n[profile default]\n",
        "[default]\nregion=us-east-1\nregion=us-west-2\n",
        "[default]\ncredential_process command\n",
        "[profile default\ncredential_process=command\n",
    ] {
        assert!(validate(CREDENTIALS, config, "default", ENDPOINT).is_err());
    }
}

#[test]
fn alternate_provider_or_endpoint_cannot_be_selected_accidentally() -> anyhow::Result<()> {
    assert!(!validate(CREDENTIALS, "", "missing", ENDPOINT)?);
    assert!(!validate(
        CREDENTIALS,
        "[default]\nendpoint_url=https://aws.amazon.com\n",
        "default",
        ENDPOINT
    )?);
    assert!(validate(
        CREDENTIALS,
        "[other]\ncredential_process=command\n",
        "default",
        ENDPOINT
    )?);
    Ok(())
}
