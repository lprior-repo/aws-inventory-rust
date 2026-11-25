use aws_wire::sign::{
    self, Body, Credentials, Error, PathMode, PayloadHeader, Region, Request, Scope, SigningName,
};
use http::{HeaderMap, Method};
use std::{error::Error as StdError, time::SystemTime};

type TestResult<T = ()> = Result<T, Box<dyn StdError>>;

fn scope_signature(scope: Scope<'_>, credentials: &Credentials) -> TestResult<String> {
    let headers = HeaderMap::new();
    let request = Request {
        method: &Method::GET,
        uri: "https://example.amazonaws.com/",
        headers: &headers,
        body: Body::Bytes(b""),
        payload_header: PayloadHeader::Omit,
    };
    let signed = sign::sign(
        credentials,
        &scope,
        SystemTime::UNIX_EPOCH,
        request,
        PathMode::Standard,
    )?;
    Ok(signed
        .get("authorization")
        .ok_or("missing authorization")?
        .to_str()?
        .to_owned())
}

#[test]
fn malformed_credential_characters_are_rejected() {
    let cases = [
        ("", "secret", None),
        ("KEY", "", None),
        ("KEY/INJECT", "secret", None),
        ("KEY", "secret\n", None),
        ("KEY", "secret", Some("")),
        ("KEY", "secret", Some("token\r\n")),
        ("KEY", "é", None),
    ];

    cases.into_iter().for_each(|(key, secret, token)| {
        assert_eq!(
            Credentials::new(key, secret, token).err(),
            Some(Error::Credentials)
        );
    });
}

#[test]
fn credentials_reject_spaces_in_secrets_and_session_tokens() {
    let cases = [("secret value", None), ("secret", Some("token value"))];

    cases.into_iter().for_each(|(secret, token)| {
        assert_eq!(
            Credentials::new("KEY", secret, token).err(),
            Some(Error::Credentials)
        );
    });
}

#[test]
fn oversized_credentials_are_rejected() {
    let key = "A".repeat(129);
    let secret = "s".repeat(129);
    let token = "t".repeat(16_385);

    assert_eq!(
        Credentials::new(&key, "s", None).err(),
        Some(Error::Credentials)
    );
    assert_eq!(
        Credentials::new("A", &secret, None).err(),
        Some(Error::Credentials)
    );
    assert_eq!(
        Credentials::new("A", "s", Some(&token)).err(),
        Some(Error::Credentials)
    );
}

#[test]
fn region_validation_rejects_invalid_lengths_and_charsets() {
    let oversized = "r".repeat(64);

    ["", "US-east-1", "a/b", "é", &oversized]
        .into_iter()
        .for_each(|value| {
            assert_eq!(Region::new(value), Err(Error::Region));
        });
}

#[test]
fn signing_name_validation_rejects_invalid_lengths_and_charsets() {
    let oversized = "s".repeat(64);

    ["", "a/b", "s p", "é", &oversized]
        .into_iter()
        .for_each(|value| {
            assert_eq!(SigningName::new(value), Err(Error::SigningName));
        });
}

#[test]
fn inclusive_scope_boundaries_are_preserved_in_authorization() -> TestResult {
    let maximum = "r".repeat(63);
    let credentials = Credentials::new("A", "s", None)?;
    ["r", &maximum, "us-east-1"]
        .into_iter()
        .try_for_each(|value| {
            let scope = Scope {
                region: Region::new(value)?,
                name: SigningName::new(value)?,
            };

            let authorization = scope_signature(scope, &credentials)?;

            assert!(authorization.contains(&format!("/{value}/{value}/aws4_request,")));
            Ok(())
        })
}

#[test]
fn maximum_credentials_are_usable_and_remain_redacted() -> TestResult {
    let key = "A".repeat(128);
    let credentials = Credentials::new(&key, &"s".repeat(128), Some(&"t".repeat(16_384)))?;
    let scope = Scope {
        region: Region::new("r")?,
        name: SigningName::new("S-3")?,
    };

    let authorization = scope_signature(scope, &credentials)?;

    assert!(authorization.contains(&format!("Credential={key}/19700101/r/S-3/aws4_request,")));
    assert_eq!(scope_signature(scope, &credentials.clone())?, authorization);
    assert_eq!(format!("{credentials:?}"), "Credentials([REDACTED])");
    Ok(())
}
