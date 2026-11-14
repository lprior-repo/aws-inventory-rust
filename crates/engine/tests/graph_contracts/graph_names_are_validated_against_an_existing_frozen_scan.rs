use super::*;

#[tokio::test]
pub(super) async fn graph_names_are_validated_against_an_existing_frozen_scan() -> TestResult {
    let folder = tempfile::tempdir()?;
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 1, "small").await?;
    let invalid = [
        String::new(),
        "é".into(),
        "graph;delete".into(),
        "graph name".into(),
        "path/name".into(),
        "contains\0nul".into(),
        "a".repeat(129),
        "b".repeat(130),
    ];
    for (index, name) in invalid.iter().enumerate() {
        let output = folder.path().join(format!("invalid-{index}.jsonl"));
        let result = graph::export_snapshot(&store, SCAN, name, &output).await;
        assert!(matches!(result, Err(GraphError::Invalid(_))));
        assert!(!output.exists());
        assert!(
            !folder
                .path()
                .join(format!("invalid-{index}.jsonl.partial"))
                .exists()
        );
    }
    for (index, name) in ["A".to_owned(), "_".into(), "-".into(), "a".repeat(128)]
        .iter()
        .enumerate()
    {
        let output = folder.path().join(format!("valid-{index}.jsonl"));
        let summary = graph::export_snapshot(&store, SCAN, name, &output).await?;
        assert_eq!(summary.graph, *name);
        let batches = read_batches(&output).await?;
        assert!(batches.iter().all(|batch| batch.graph == *name));
        assert_eq!(
            batches.last().map(|batch| batch.stage.as_str()),
            Some("end")
        );
    }
    store.close().await;
    Ok(())
}
