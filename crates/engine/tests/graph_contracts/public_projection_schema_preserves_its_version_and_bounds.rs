use super::*;

#[test]
pub(super) fn public_projection_schema_preserves_its_version_and_bounds() {
    assert_eq!(
        graph::graph_batch_schema_version(),
        json!({
            "version": 1,
            "max_rows": 256,
            "max_query_bytes": 2_097_152,
        })
    );
}
