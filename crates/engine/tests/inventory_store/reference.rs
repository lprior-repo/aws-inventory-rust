use inventory_engine::inventory::SweepId;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::support::*;

#[tokio::test]
async fn given_a_target_identity_value_changes_when_an_old_reference_exists_then_its_edge_is_retracted_and_reference_kept()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store
        .seed_work(work("reference-before", "us-east-1")?)
        .await?;
    let first_sweep = SweepId::new("reference-before".to_owned())?;
    let first = store
        .claim(&first_sweep, 100, 20)
        .await?
        .ok_or("missing initial lease")?;
    let mut initial = page(
        first,
        vec![
            observation("target", "old-name")?,
            observation("source", "source-name")?,
        ],
        true,
        None,
        101,
    );
    initial.unresolved.push(unresolved("source", "old-name")?);
    store.commit_page(initial).await?;

    store
        .seed_work(work("reference-after", "us-east-1")?)
        .await?;
    let second_sweep = SweepId::new("reference-after".to_owned())?;
    let second = store
        .claim(&second_sweep, 200, 20)
        .await?
        .ok_or("missing update lease")?;
    store
        .commit_page(page(
            second,
            vec![observation("target", "new-name")?],
            false,
            Some(vec![8]),
            201,
        ))
        .await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let stale_edges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM edges WHERE from_id='source' AND to_id='target'")
            .fetch_one(&pool)
            .await?;
    let retained_references: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs WHERE from_id='source'")
            .fetch_one(&pool)
            .await?;
    pool.close().await;

    assert_eq!(stale_edges, 0);
    assert_eq!(retained_references, 1);
    Ok(())
}
