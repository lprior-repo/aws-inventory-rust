 

use std::error::Error;

use aws_emulator_embedded::{Authentication, AwsEmulatorEmbedded, EmbeddedOptions, Telemetry};
use inventory_emulator_embedded::{BOOT_STACK_SIZE, WORKER_STACK_SIZE};

type TestResult = Result<(), Box<dyn Error + Send + Sync>>;

 
fn emulator() -> AwsEmulatorEmbedded {
    AwsEmulatorEmbedded::new(
        EmbeddedOptions::default()
            .authentication(Authentication::On)
            .telemetry(Telemetry::Off),
    )
}

 
async fn client(emulator: &AwsEmulatorEmbedded) -> aws_sdk_s3::Client {
    let shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(emulator.aws_credentials_provider())
        .http_client(emulator.http_client())
        .load()
        .await;
    aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::from(&shared)
            .force_path_style(true)
            .build(),
    )
}

 
fn run<F, Fut>(operation: F) -> TestResult
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = TestResult>,
{
    std::thread::Builder::new()
        .name("emulator-boot".to_owned())
        .stack_size(BOOT_STACK_SIZE)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(WORKER_STACK_SIZE)
                .enable_all()
                .build()?;
            runtime.block_on(operation())
        })?
        .join()
        .map_err(|_| std::io::Error::other("emulator boot thread panicked"))?
}

#[test]
fn given_created_bucket_when_listed_then_exact_name_is_returned() -> TestResult {
    run(|| async {
        let emulator = emulator();
        let client = client(&emulator).await;
        client
            .create_bucket()
            .bucket("embedded-seam-check")
            .send()
            .await?;
        let listed = client.list_buckets().send().await?;
        let names: Vec<&str> = listed
            .buckets()
            .iter()
            .filter_map(|bucket| bucket.name())
            .collect();
        assert_eq!(names, vec!["embedded-seam-check"]);
        Ok(())
    })
}

#[test]
fn given_two_emulators_when_one_seeds_a_bucket_then_state_is_not_shared() -> TestResult {
    run(|| async {
        let first = emulator();
        let first_client = client(&first).await;
        first_client
            .create_bucket()
            .bucket("isolated-first")
            .send()
            .await?;
        let second = emulator();
        let second_client = client(&second).await;
        let listed = second_client.list_buckets().send().await?;
        assert!(
            listed.buckets().is_empty(),
            "separate instances must not share resource state"
        );
        Ok(())
    })
}
