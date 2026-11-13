//! environment responsibilities for emulator_snapshot.
use super::*;

pub(super) fn fail(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::other(message.into()).into()
}

pub(super) fn required_env(name: &str) -> TestResult<String> {
    std::env::var(name).map_err(|_| {
        fail(format!(
            "required prepared-emulator variable {name} is absent or not Unicode"
        ))
    })
}

pub(super) fn bounded_file(path: &Path, limit: u64) -> TestResult<Vec<u8>> {
    if !path.is_absolute() {
        return Err(fail("prepared fixture paths must be absolute"));
    }
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(fail("prepared fixture must be a regular file"));
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())? > limit {
        return Err(fail("prepared fixture file exceeded its byte budget"));
    }
    Ok(bytes)
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", base16ct::HexDisplay(&Sha256::digest(bytes)))
}

pub(super) fn verify_digest(bytes: &[u8], expected: &str, label: &str) -> TestResult {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(fail(format!("{label} SHA-256 is malformed")));
    }
    if digest(bytes) != expected.to_ascii_lowercase() {
        return Err(fail(format!(
            "{label} SHA-256 does not match the frozen receipt"
        )));
    }
    Ok(())
}

pub(super) fn ini_default(bytes: &[u8]) -> TestResult<BTreeMap<String, String>> {
    let text = std::str::from_utf8(bytes)?;
    let mut fields = BTreeMap::new();
    let mut default_section = false;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            if line != "[default]" || default_section {
                return Err(fail("only one isolated default AWS profile is allowed"));
            }
            default_section = true;
            continue;
        }
        if !default_section {
            return Err(fail(
                "AWS fixture configuration must start with its default section",
            ));
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| fail("malformed fixture configuration line"))?;
        if fields
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(fail("duplicate key in fixture configuration"));
        }
    }
    if !default_section {
        return Err(fail("fixture configuration lacks a default section"));
    }
    Ok(fields)
}

pub(super) fn verify_environment() -> TestResult {
    if required_env("INVENTORY_EMULATOR_TEST")? != "1"
        || required_env("AWS_EC2_METADATA_DISABLED")? != "true"
        || required_env("AWS_REGION")? != REGION
        || required_env("AWS_DEFAULT_REGION")? != REGION
    {
        return Err(fail(
            "explicit local emulator opt-in, disabled IMDS and the fixture region are required",
        ));
    }
    for name in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_PROFILE",
        "AWS_DEFAULT_PROFILE",
        "AWS_ROLE_ARN",
        "AWS_WEB_IDENTITY_TOKEN_FILE",
        "AWS_CONTAINER_CREDENTIALS_FULL_URI",
        "AWS_CONTAINER_CREDENTIALS_RELATIVE_URI",
        "AWS_IGNORE_CONFIGURED_ENDPOINT_URLS",
    ] {
        if std::env::var_os(name).is_some() {
            return Err(fail(format!(
                "ambient credential/endpoint override {name} is forbidden"
            )));
        }
    }
    if std::env::vars_os().any(|(name, _)| name.to_string_lossy().starts_with("AWS_ENDPOINT_URL_"))
    {
        return Err(fail("service-specific endpoint overrides are forbidden"));
    }
    let endpoint = required_env("AWS_ENDPOINT_URL")?;
    let port = endpoint
        .strip_prefix("https://127.0.0.1:")
        .ok_or_else(|| fail("emulator endpoint must be HTTPS on literal IPv4 loopback"))?
        .parse::<u16>()?;
    if port == 0 {
        return Err(fail("emulator endpoint must have a nonzero port"));
    }
    let credentials = PathBuf::from(required_env("AWS_SHARED_CREDENTIALS_FILE")?);
    if credentials.file_name().and_then(|name| name.to_str()) != Some("emulator_credentials") {
        return Err(fail(
            "credentials must be the emulator's generated credentials file",
        ));
    }
    let directory = credentials
        .parent()
        .ok_or_else(|| fail("credentials have no parent directory"))?;
    if directory.file_name().and_then(|name| name.to_str()) != Some(port.to_string().as_str()) {
        return Err(fail(
            "generated credentials do not belong to the configured emulator port",
        ));
    }
    let ca = PathBuf::from(required_env("SSL_CERT_FILE")?);
    if ca != directory.join("ca.pem") || bounded_file(&ca, 1024 * 1024)?.is_empty() {
        return Err(fail(
            "Rust TLS must verify this emulator instance's generated CA",
        ));
    }
    let configuration = PathBuf::from(required_env("AWS_CONFIG_FILE")?);
    let profile = ini_default(&bounded_file(&configuration, 16 * 1024)?)?;
    for (key, value) in &profile {
        match key.as_str() {
            "region" if value == REGION => {}
            "endpoint_url" if value == &endpoint => {}
            "ca_bundle" if Path::new(value) == ca => {}
            _ => {
                return Err(fail(format!(
                    "unexpected key or scope in emulator configuration: {key}"
                )));
            }
        }
    }
    if profile.get("endpoint_url") != Some(&endpoint)
        || profile.get("region").map(String::as_str) != Some(REGION)
    {
        return Err(fail(
            "isolated SDK configuration lacks the required endpoint or region",
        ));
    }
    let keys = ini_default(&bounded_file(&credentials, 16 * 1024)?)?;
    if keys.len() != 2
        || !keys
            .get("aws_access_key_id")
            .is_some_and(|value| value.starts_with("EMULATOR") && value.len() <= 128)
        || !keys
            .get("aws_secret_access_key")
            .is_some_and(|value| !value.is_empty() && value.len() <= 256)
    {
        return Err(fail(
            "credentials are not the expected bounded emulator-generated base keys",
        ));
    }
    Ok(())
}
