use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use aws_sdk_ecs::types::{
    Compatibility, CpuArchitecture, DockerVolumeConfiguration, EfsVolumeConfiguration,
    EnvironmentFile, EnvironmentFileType, FSxWindowsFileServerAuthorizationConfig,
    FSxWindowsFileServerVolumeConfiguration, FirelensConfiguration, FirelensConfigurationType,
    HealthCheck, KeyValuePair, LogDriver, MountPoint, NetworkMode, OsFamily, PortMapping,
    ProxyConfiguration, RepositoryCredentials, RuntimePlatform, S3FilesVolumeConfiguration,
    TaskDefinition,
};
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::*;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;
const ARN: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:7";
const SECOND: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:8";
const ROLE: &str = "arn:aws:iam::222222222222:role/application/web";
const SECRET_REF: &str = "arn:aws:secretsmanager:us-east-1:111111111111:secret:database-ref";

fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(32);
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

fn definition(arn: &str, revision: i32) -> TaskDefinition {
    TaskDefinition::builder()
        .task_definition_arn(arn)
        .family("web")
        .revision(revision)
        .status(TaskDefinitionStatus::Active)
        .build()
}

fn output(definition: TaskDefinition) -> DescribeTaskDefinitionOutput {
    DescribeTaskDefinitionOutput::builder()
        .task_definition(definition)
        .build()
}

fn resource(body: &PageBody) -> Result<&Resource, TestError> {
    body.resources
        .first()
        .ok_or_else(|| "missing task definition".into())
}

fn drain(
    receiver: &mut mpsc::Receiver<CollectionEvent>,
) -> Result<Vec<CollectionEvent>, TestError> {
    let mut events = Vec::new();
    for _ in 0..32 {
        match receiver.try_recv() {
            Ok(event) => events.push(event),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                return Ok(events);
            }
        }
    }
    Err("synthetic event fixture exceeded its bound".into())
}

mod empty_list_page_with_token_continues_to_a_revision_and_terminal_page;
mod environment_and_unstructured_option_values_are_not_retained;
mod revision_identity_and_architecture_fields_use_returned_arn;
mod volume_references_aggregate_and_preserve_authoritative_or_assumed_owners;
