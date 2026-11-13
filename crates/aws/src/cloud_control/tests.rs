use super::*;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;

fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(64);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages,
            page_size: 100,
        },
        receiver,
    ))
}

fn descriptor(name: &str) -> Result<ResourceType, TestError> {
    catalog()?
        .into_iter()
        .find(|descriptor| descriptor.type_name == name)
        .ok_or_else(|| format!("missing fixture type {name}").into())
}

fn description(id: &str, properties: Value) -> ResourceDescription {
    ResourceDescription::builder()
        .identifier(id)
        .properties(properties.to_string())
        .build()
}

fn get(description: ResourceDescription) -> GetResourceOutput {
    GetResourceOutput::builder()
        .resource_description(description)
        .build()
}

fn first(body: &PageBody) -> Result<&Resource, TestError> {
    body.resources
        .first()
        .ok_or_else(|| "missing resource".into())
}

fn drain(
    receiver: &mut mpsc::Receiver<CollectionEvent>,
) -> Result<Vec<CollectionEvent>, TestError> {
    let mut events = Vec::new();
    for _ in 0..64 {
        match receiver.try_recv() {
            Ok(event) => events.push(event),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                return Ok(events);
            }
        }
    }
    Err("fixture event capacity exceeded".into())
}

mod absent_list_properties_still_enriches_with_the_exact_composite_identifier;
mod catalog_has_397_reviewed_regional_read_list_types_and_safe_columns;
mod read_visible_environment_commands_parameter_values_and_firehose_tokens_are_redacted;
mod unsupported_list_and_closed_consumer_never_claim_empty_success;
mod unsupported_type_list_is_accounted_without_failing_the_scan;
