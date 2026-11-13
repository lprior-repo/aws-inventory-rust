use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::{GraphExportCheckpoint, MAX_QUERY_ROWS, Store, StoreError};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult = Result<(), StoreError>;

fn scope(collector: &str, source: Source) -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        collector: collector.to_owned(),
        parent_id: None,
        source,
    }
}

fn resource(id: &str, source: Source, observed_at: i64) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "aws_ec2_instance".to_owned(),
        name: Some("worker".to_owned()),
        properties: json!({"instance_id": id, "instance_type": "m7i.large", "state": "running"}),
        tags: BTreeMap::from([("env".to_owned(), "production".to_owned())]),
        source,
        observed_at,
    }
}

fn page(collector: &str, source: Source, resources: Vec<Resource>) -> CollectionPage {
    CollectionPage {
        scope: scope(collector, source),
        page_number: 1,
        next_token: None,
        resources,
        edges: Vec::new(),
    }
}

fn relationship() -> Edge {
    Edge {
        from_id: "i-1".to_owned(),
        to_id: "vpc-unresolved".to_owned(),
        relation: "IN_VPC".to_owned(),
        source: Source::Direct,
        observed_at: 10,
        properties: json!({}),
    }
}

#[path = "store/cancelled_query_keeps_runtime_responsive_and_reader_reusable.rs"]
mod cancelled_query_keeps_runtime_responsive_and_reader_reusable;
#[path = "store/changed_replay_cannot_overwrite_committed_observation.rs"]
mod changed_replay_cannot_overwrite_committed_observation;
#[path = "store/duplicate_edge_after_first_sql_batch_rolls_back_resources_too.rs"]
mod duplicate_edge_after_first_sql_batch_rolls_back_resources_too;
#[path = "store/duplicate_resource_after_first_sql_batch_rolls_back_entire_page.rs"]
mod duplicate_resource_after_first_sql_batch_rolls_back_entire_page;
#[path = "store/edge_failure_also_rolls_back_resources_and_checkpoint.rs"]
mod edge_failure_also_rolls_back_resources_and_checkpoint;
#[path = "store/empty_resource_name_preserves_valid_aws_tag_value.rs"]
mod empty_resource_name_preserves_valid_aws_tag_value;
#[path = "store/empty_terminal_page_proves_empty_scope_but_unfinished_page_does_not.rs"]
mod empty_terminal_page_proves_empty_scope_but_unfinished_page_does_not;
#[path = "store/equal_timestamp_source_ties_ignore_page_arrival_order.rs"]
mod equal_timestamp_source_ties_ignore_page_arrival_order;
#[path = "store/exact_page_replay_is_idempotent_even_after_finalization.rs"]
mod exact_page_replay_is_idempotent_even_after_finalization;
#[path = "store/failed_scopes_are_durable_idempotent_and_prevent_complete_status.rs"]
mod failed_scopes_are_durable_idempotent_and_prevent_complete_status;
#[path = "store/file_reader_denies_cte_writes_and_preserves_bound_values.rs"]
mod file_reader_denies_cte_writes_and_preserves_bound_values;
#[path = "store/graph_checkpoints_are_monotonic_and_require_a_frozen_scan.rs"]
mod graph_checkpoints_are_monotonic_and_require_a_frozen_scan;
#[path = "store/invalid_scan_lifecycle_does_not_reopen_terminal_scans.rs"]
mod invalid_scan_lifecycle_does_not_reopen_terminal_scans;
#[path = "store/observation_failure_rolls_back_page_checkpoint_edges_and_other_observations.rs"]
mod observation_failure_rolls_back_page_checkpoint_edges_and_other_observations;
#[path = "store/pages_require_contiguous_order_and_reject_continuation_cycles.rs"]
mod pages_require_contiguous_order_and_reject_continuation_cycles;
#[path = "store/partial_snapshot_never_erases_previous_resources_or_edges.rs"]
mod partial_snapshot_never_erases_previous_resources_or_edges;
#[path = "store/preferred_source_and_latest_observation_within_source_are_deterministic.rs"]
mod preferred_source_and_latest_observation_within_source_are_deterministic;
#[path = "store/query_bounds_native_columns_and_decoded_cells.rs"]
mod query_bounds_native_columns_and_decoded_cells;
#[path = "store/query_caps_rows_columns_output_and_instruction_work.rs"]
mod query_caps_rows_columns_output_and_instruction_work;
#[path = "store/query_denies_writes_pragma_transactions_and_external_side_effects.rs"]
mod query_denies_writes_pragma_transactions_and_external_side_effects;
#[path = "store/query_single_statement_check_understands_quotes_and_comments.rs"]
mod query_single_statement_check_understands_quotes_and_comments;
#[path = "store/quoted_identifiers_and_values_are_bound_in_ingestion_paths.rs"]
mod quoted_identifiers_and_values_are_bound_in_ingestion_paths;
#[path = "store/reopen_preserves_atomic_page_and_resume_token.rs"]
mod reopen_preserves_atomic_page_and_resume_token;
#[path = "store/separate_memory_stores_do_not_share_data.rs"]
mod separate_memory_stores_do_not_share_data;
#[path = "store/sql_views_expose_selected_configuration_and_provenance.rs"]
mod sql_views_expose_selected_configuration_and_provenance;
#[path = "store/streaming_exports_propagate_consumer_failure.rs"]
mod streaming_exports_propagate_consumer_failure;
#[path = "store/successful_page_roundtrips_all_rows_across_multiple_sql_batches.rs"]
mod successful_page_roundtrips_all_rows_across_multiple_sql_batches;
