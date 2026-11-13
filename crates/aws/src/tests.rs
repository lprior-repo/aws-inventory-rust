use super::{AwsScanError, AwsScanOptions, scan, validate_region};
use futures_util::FutureExt;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use std::sync::Arc;

#[test]
fn selectors_include_new_groups_without_duplicate_collectors() -> Result<(), AwsScanError> {
    let mut options = AwsScanOptions {
        profile: None,
        regions: vec!["us-east-1".into()],
        services: vec![],
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: false,
        assume_role: None,
        cloud_control: false,
        selection: inventory_core::Selection::Types(Vec::new()),
        max_pages: 1,
        page_size: 100,
    };
    let all = options.validate()?;
    assert_eq!(all.len(), 16);
    assert!(all.contains("network"));
    assert!(all.contains("elbv2"));
    options.services = vec!["ec2".into(), "vpc".into(), "network".into()];
    assert_eq!(
        options.validate()?,
        std::collections::BTreeSet::from(["ec2".to_owned(), "network".to_owned(),])
    );
    options.services = vec!["network".into()];
    assert_eq!(
        options.validate()?,
        std::collections::BTreeSet::from(["network".to_owned(),])
    );
    options.services = vec!["elb".into(), "elbv2".into(), "load-balancing".into()];
    assert_eq!(
        options.validate()?,
        std::collections::BTreeSet::from(["elbv2".to_owned(),])
    );
    options.services = vec!["all".into(), "unknown-service".into()];
    assert!(options.validate().is_err());
    Ok(())
}

#[test]
fn supplemental_only_selection_does_not_enable_native_collectors() -> Result<(), AwsScanError> {
    let mut options = AwsScanOptions {
        profile: None,
        regions: vec![],
        services: vec!["config".into(), "resource-explorer".into()],
        config_aggregator: Some("fixture-aggregator".into()),
        resource_explorer_view: Some("fixture-view".into()),
        discover_sources: false,
        assume_role: None,
        cloud_control: false,
        selection: inventory_core::Selection::Types(Vec::new()),
        max_pages: 1,
        page_size: 100,
    };
    assert!(options.validate()?.is_empty());
    options.config_aggregator = None;
    assert!(options.validate().is_err());
    Ok(())
}

#[test]
fn rejects_unbounded_options() {
    let options = AwsScanOptions {
        profile: None,
        regions: vec![],
        services: vec![],
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: false,
        assume_role: None,
        cloud_control: false,
        selection: inventory_core::Selection::Types(Vec::new()),
        max_pages: 10_001,
        page_size: 100,
    };
    assert!(options.validate().is_err());
    assert!(validate_region("us-east-1").is_ok());
    assert!(validate_region("us-east-1\n").is_err());
}

#[test]
fn pre_cancelled_scan_returns_before_sdk_setup() -> Result<(), Box<dyn std::error::Error>> {
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
    executor.cancel();
    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    let options = AwsScanOptions {
        profile: None,
        regions: vec!["us-east-1".into()],
        services: vec!["ec2".into()],
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: false,
        assume_role: None,
        cloud_control: false,
        selection: inventory_core::Selection::Types(Vec::new()),
        max_pages: 1,
        page_size: 100,
    };
    let outcome = scan(options, Arc::clone(&executor), sender).now_or_never();
    assert!(matches!(
        outcome,
        Some(Err(AwsScanError::Api(ApiError::Cancelled)))
    ));
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}
