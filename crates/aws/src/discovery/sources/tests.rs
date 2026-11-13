use super::*;
use aws_config::BehaviorVersion;
use aws_types::{SdkConfig, region::Region};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};

fn config() -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .build()
}
fn client<'a>(config: &'a SdkConfig, executor: &'a ApiExecutor) -> DiscoveryClient<'a> {
    DiscoveryClient {
        config,
        executor,
        account: "123456789012",
        partition: "aws",
        region: "us-east-1",
        max_pages: 10,
        page_size: 100,
    }
}

#[test]
fn unique_source_selection_never_uses_partial_or_ambiguous_evidence() {
    assert_eq!(
        select_unique(vec![], true, "fixture"),
        SourceSelection::Absent
    );
    assert!(matches!(
        select_unique(vec!["one".into()], true, "fixture"),
        SourceSelection::Selected { .. }
    ));
    assert!(matches!(
        select_unique(vec!["one".into(), "two".into()], true, "fixture"),
        SourceSelection::Ambiguous { .. }
    ));
    assert!(matches!(
        select_unique(vec!["one".into()], false, "fixture"),
        SourceSelection::Unavailable { .. }
    ));
}

#[test]
fn regional_default_is_selected_only_when_observed_in_complete_listing() {
    let candidates = vec!["one".into(), "two".into()];
    assert_eq!(
        select_view_candidate(&candidates, Some("two")).selected_identifier(),
        Some("two")
    );
    assert!(matches!(
        select_view_candidate(&candidates, Some("missing")),
        SourceSelection::Unavailable { .. }
    ));
    assert!(matches!(
        select_view_candidate(&candidates, None),
        SourceSelection::Ambiguous { .. }
    ));
}

#[test]
fn sources_are_account_and_region_scoped() -> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    assert!(
        validate_scoped_arn(
            &client,
            "arn:aws:resource-explorer-2:us-east-1:123456789012:view/all/id",
            "resource-explorer-2",
            "view/"
        )
        .is_ok()
    );
    assert!(
        validate_scoped_arn(
            &client,
            "arn:aws:resource-explorer-2:us-west-2:123456789012:view/all/id",
            "resource-explorer-2",
            "view/"
        )
        .is_err()
    );
    assert!(
        validate_scoped_arn(
            &client,
            "arn:aws:resource-explorer-2:us-east-1:223456789012:view/all/id",
            "resource-explorer-2",
            "view/"
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn view_filter_values_are_never_persisted() -> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let arn = "arn:aws:resource-explorer-2:us-east-1:123456789012:view/all/id";
    let view = View::builder()
        .view_arn(arn)
        .owner(client.account)
        .filters(
            aws_sdk_resourceexplorer2::types::SearchFilter::builder()
                .filter_string("tag:token=DO_NOT_PERSIST")
                .build()?,
        )
        .build();
    let mapped = map_view(&client, arn, view)?;
    assert!(mapped.has_filter);
    assert!(!serde_json::to_string(&mapped)?.contains("DO_NOT_PERSIST"));
    Ok(())
}

#[test]
fn config_aggregator_metadata_retains_declared_scope_and_filter_presence()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let source = aws_sdk_config::types::AccountAggregationSource::builder()
        .account_ids("123456789012")
        .aws_regions("us-east-1")
        .build()?;
    let value = ConfigurationAggregator::builder()
        .configuration_aggregator_name("audit")
        .configuration_aggregator_arn(
            "arn:aws:config:us-east-1:123456789012:config-aggregator/example",
        )
        .account_aggregation_sources(source)
        .build();
    let mapped = map_aggregator(&client, value)?;
    assert_eq!(mapped.name, "audit");
    assert!(!mapped.organization_source);
    assert_eq!(
        mapped
            .account_sources
            .first()
            .map(|source| source.account_ids.as_slice()),
        Some(["123456789012".to_owned()].as_slice())
    );
    Ok(())
}

#[test]
fn view_metadata_must_match_the_selected_identity() -> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let arn = "arn:aws:resource-explorer-2:us-east-1:123456789012:view/other/id";
    assert!(map_view(&client, "different", View::builder().view_arn(arn).build()).is_err());
    assert!(
        map_view(
            &client,
            arn,
            View::builder().view_arn(arn).owner("223456789012").build()
        )
        .is_err()
    );
    Ok(())
}
