use super::*;
use aws_sdk_cloudcontrol::operation::list_resources::ListResourcesError;
use aws_sdk_cloudcontrol::types::error::TypeNotFoundException;
use aws_sdk_sts::{config::http::HttpResponse, error::SdkError};
use aws_smithy_types::body::SdkBody;
use inventory_core::{CollectionEvent, ScanStatus};
use inventory_engine::{
    limiter::{ApiExecutor, ExecutorConfig},
    store::Store,
};
use tokio::sync::mpsc;

type TestError = Box<dyn std::error::Error>;

fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(16);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages: 2,
            page_size: 2,
        },
        receiver,
    ))
}

async fn feed(
    store: &Store,
    scan: &str,
    receiver: &mut mpsc::Receiver<CollectionEvent>,
) -> Result<(), TestError> {
    while let Ok(event) = receiver.try_recv() {
        match event {
            CollectionEvent::Page(page) => store.apply_page(scan, page).await?,
            CollectionEvent::Failure(failure) => store.record_failure(scan, &failure).await?,
        }
    }
    Ok(())
}

#[test]
fn selection_covers_exactly_the_list_capable_catalog_rows() -> Result<(), TestError> {
    let (summary, rows) = selection::catalog()?;
    let descriptors = list_descriptors()?;
    assert_eq!(descriptors.len(), summary.list_capable);
    assert_eq!(
        summary
            .list_capable
            .saturating_add(summary.parent_required)
            .saturating_add(summary.unsupported),
        rows.len()
    );
    let mut names = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut views = BTreeSet::new();
    for descriptor in &descriptors {
        assert!(descriptor.list_supported && !descriptor.list_requires_model);
        assert!(names.insert(descriptor.type_name.clone()));
        assert!(kinds.insert(descriptor.kind.clone()));
        assert!(views.insert(descriptor.view_name.clone()));
        assert!(descriptor.kind.len() <= 128 && descriptor.view_name.len() <= 128);
    }
    for row in &rows {
        let selected = names.contains(&row.type_name);
        assert_eq!(
            selected,
            row.capability == "list",
            "row {} was not classified consistently",
            row.type_name
        );
    }
    let via_cli = crate::cloud_control::validate_selection(&Selection::Universal)?;
    assert_eq!(via_cli.len(), descriptors.len());
    assert!(
        Selection::parse(&["all".to_owned(), "AWS::EC2::Instance".to_owned(),]).is_err(),
        "`all` mixed with explicit types must be rejected"
    );
    Ok(())
}

#[test]
fn synthesized_descriptors_apply_the_global_sensitive_policy() -> Result<(), TestError> {
    let descriptor = list_descriptors()?
        .into_iter()
        .find(|descriptor| descriptor.type_name == "AWS::Amplify::App")
        .ok_or("AWS::Amplify::App must be list-capable")?;
    assert!(descriptor.sensitive_names.contains("environmentvariables"));
    assert!(descriptor.sensitive_names.contains("password"));
    let raw = json!({
        "EnvironmentVariables": {"DATABASE_URL": "postgres://fixture"},
        "Password": "hunter2",
        "Name": "safe-name"
    })
    .to_string();
    let (sanitized, redacted) = sanitized_properties(&descriptor, Some(&raw))?;
    assert!(redacted.iter().any(|path| path == "/EnvironmentVariables"));
    assert!(redacted.iter().any(|path| path == "/Password"));
    assert!(sanitized.get("EnvironmentVariables").is_none());
    assert!(sanitized.get("Password").is_none());
    assert_eq!(
        sanitized.get("Name").and_then(Value::as_str),
        Some("safe-name")
    );
    Ok(())
}

#[test]
fn report_partitions_every_catalog_row_with_a_digest() -> Result<(), TestError> {
    let report = report()?;
    assert_eq!(report.total, 1_881);
    assert_eq!(
        report
            .list_capable
            .saturating_add(report.parent_required)
            .saturating_add(report.unsupported),
        report.total
    );
    assert_eq!(report.skipped_parent_required.len(), report.parent_required);
    assert_eq!(report.skipped_unsupported.len(), report.unsupported);
    assert_eq!(report.catalog_digest, digest());
    assert_eq!(report.catalog_digest.len(), 64);
    Ok(())
}

#[test]
fn type_not_found_is_a_provider_boundary_not_generic_api_error() -> Result<(), TestError> {
    let response = HttpResponse::new(400_u16.try_into()?, SdkBody::empty());
    let error: SdkError<ListResourcesError> = SdkError::service_error(
        ListResourcesError::TypeNotFoundException(
            TypeNotFoundException::builder()
                .message("fixture absent type")
                .build(),
        ),
        response,
    );
    match super::outcomes::list_error(error) {
        ApiError::UnsupportedType(message) => {
            assert!(message.contains("TypeNotFoundException"));
        }
        other => return Err(format!("unexpected classification: {other:?}").into()),
    }
    let category = error_category(&AwsScanError::Api(ApiError::UnsupportedType(
        "fixture".into(),
    )));
    assert_eq!(category, "unsupported_type");
    Ok(())
}

#[tokio::test]
async fn unsupported_lists_complete_while_real_errors_stay_partial() -> Result<(), TestError> {
    let store = Store::open(":memory:").await?;
    let descriptor = list_descriptors()?
        .into_iter()
        .next()
        .ok_or("empty universal selection")?;

    let (ctx, mut receiver) = context()?;
    super::outcomes::collect_type(&ctx, &descriptor, |_| async {
        Err(ApiError::UnsupportedType(
            "TypeNotFoundException: fixture".into(),
        ))
    })
    .await?;
    let scan = "scan-unsupported";
    store.begin_scan(scan, 1).await?;
    feed(&store, scan, &mut receiver).await?;
    store.finish_scan(scan, ScanStatus::Complete, 2).await?;
    assert_eq!(store.scan_status(scan).await?, ScanStatus::Complete);
    let outcomes = store.universal_outcomes(scan).await?;
    assert_eq!(outcomes.attempted, 1);
    assert_eq!(outcomes.succeeded, 0);
    assert_eq!(outcomes.unsupported, 1);
    assert_eq!(outcomes.failed, 0);
    assert_eq!(
        outcomes.attempted,
        outcomes
            .succeeded
            .saturating_add(outcomes.unsupported)
            .saturating_add(outcomes.failed)
    );

    let (ctx, mut receiver) = context()?;
    super::outcomes::collect_type(&ctx, &descriptor, |_| async {
        Err(ApiError::Denied("fixture denial".into()))
    })
    .await?;
    let scan = "scan-denied";
    store.begin_scan(scan, 3).await?;
    feed(&store, scan, &mut receiver).await?;
    store.finish_scan(scan, ScanStatus::Complete, 4).await?;
    assert_eq!(store.scan_status(scan).await?, ScanStatus::Partial);
    let outcomes = store.universal_outcomes(scan).await?;
    assert_eq!(outcomes.attempted, 1);
    assert_eq!(outcomes.unsupported, 0);
    assert_eq!(outcomes.failed, 1);
    Ok(())
}
