//! Opt-in TLS-upstream parity: real S3 SDK signing, transport and error decoding.
use aws_sdk_s3::config::{Credentials, Region, retry::RetryConfig};
use inventory_harness::{
    emulator::Emulator,
    proxy::{BodyStyle, Config, Fault, Proxy, Schedule, Scope},
};
use std::error::Error;
use tokio::io::AsyncReadExt;

type TestResult = Result<(), Box<dyn Error>>;
const MAX_CREDENTIAL_BYTES: u64 = 16_384;
const EMULATOR_IDLE_SECONDS: u64 = 300;

#[test]
#[ignore = "requires INVENTORY_EMULATOR_BINARY: real emulator TLS upstream and S3 SDK"]
fn given_tls_emulator_when_s3_uses_proxy_then_forwarding_and_scheduled_throttle_decode()
-> TestResult {
    // Given an owned emulator and its own generated trust/credential material.
    let work = tempfile::tempdir()?;
    let emulator = Emulator::start(
        &super::emulator_binary()?,
        work.path(),
        EMULATOR_IDLE_SECONDS,
    )?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    // When the SDK uses the HTTP proxy, then outbound TLS and error decoding work.
    runtime.block_on(exercise(&emulator))?;
    emulator.stop()?;
    Ok(())
}

async fn exercise(emulator: &Emulator) -> TestResult {
    let credentials = credentials(emulator.credentials()).await?;
    let key = credentials.access_key_id().to_owned();
    let mut config = Config::new(emulator.endpoint(), 7);
    config.ca_bundle = Some(emulator.ca_bundle().to_owned());
    config.default_style = BodyStyle::RestXml;
    clean_forwarding(&config, &credentials).await?;
    config.schedule.push(Schedule {
        scope: Scope::new(&key, "us-east-1", "s3")?,
        ordinal: 0,
        fault: Fault::Throttle,
    });
    throttled(config, credentials).await
}

fn client(endpoint: &str, credentials: Credentials) -> aws_sdk_s3::Client {
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version_latest()
        .credentials_provider(credentials)
        .region(Region::new("us-east-1"))
        .endpoint_url(endpoint)
        .force_path_style(true)
        .retry_config(RetryConfig::standard().with_max_attempts(1))
        .build();
    aws_sdk_s3::Client::from_conf(config)
}

async fn credentials(path: &std::path::Path) -> Result<Credentials, Box<dyn Error>> {
    let mut text = String::new();
    tokio::fs::File::open(path)
        .await?
        .take(MAX_CREDENTIAL_BYTES.saturating_add(1))
        .read_to_string(&mut text)
        .await?;
    if u64::try_from(text.len())? > MAX_CREDENTIAL_BYTES {
        return Err("credential file bound".into());
    }
    let access = ini(&text, "aws_access_key_id").ok_or("missing access key")?;
    let secret = ini(&text, "aws_secret_access_key").ok_or("missing secret key")?;
    Ok(Credentials::new(
        access,
        secret,
        ini(&text, "aws_session_token").map(str::to_owned),
        None,
        "owned-emulator",
    ))
}
fn ini<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim())
}

async fn clean_forwarding(config: &Config, credentials: &Credentials) -> TestResult {
    let clean = Proxy::start(config.clone()).await?;
    let output = client(clean.endpoint(), credentials.clone())
        .list_buckets()
        .send()
        .await?;
    assert_eq!(
        output
            .buckets()
            .iter()
            .filter_map(|bucket| bucket.name())
            .count(),
        output.buckets().len()
    );
    assert_eq!(clean.snapshot()?.counters.forwarded, 1);
    clean.shutdown().await?;
    Ok(())
}

async fn throttled(config: Config, credentials: Credentials) -> TestResult {
    let faulty = Proxy::start(config).await?;
    let result = client(faulty.endpoint(), credentials)
        .list_buckets()
        .send()
        .await;
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err("scheduled throttle unexpectedly succeeded".into()),
    };
    let service = error
        .as_service_error()
        .ok_or("throttle was not a service error")?;
    assert_eq!(service.meta().code(), Some("SlowDown"));
    assert_eq!(
        error
            .raw_response()
            .map(|response| response.status().as_u16()),
        Some(503)
    );
    assert_eq!(faulty.snapshot()?.counters.forwarded, 0);
    faulty.shutdown().await?;
    Ok(())
}
