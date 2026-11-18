use super::*;
use fixture::{catalog_bytes_with_refresh, frozen, page_with_input, page_with_sink, pool};
use inventory_core::record::Key;

fn input(max_results: i64) -> TestResult<Record> {
    let fields = catalog_tool::smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/field-ids.txt"
    )))?;
    let field = fields
        .id("MaxResults")
        .ok_or("native MaxResults field absent")?;
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(field),
        Value::Int(max_results),
    )])))?)
}

#[tokio::test]
async fn given_fresh_source_x_grant_when_only_its_codec_is_copied_into_changed_payload_or_input_y_then_y_cannot_activate_x()
-> TestResult {
    let bytes = catalog_bytes_with_refresh()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for change_payload in [false, true] {
        let (directory, store) = store(&catalog).await?;
        store
            .commit_page(
                page_with_input(
                    &store,
                    &catalog,
                    "metadata",
                    "source-x",
                    1,
                    (Some(SINK), input(1)?),
                )
                .await?,
            )
            .await?;
        let sql = pool(&directory).await?;
        let row = sqlx::query(
            "SELECT source_key,source_authority,relations FROM resource_facets WHERE id=?1",
        )
        .bind(LINK)
        .fetch_one(&sql)
        .await?;
        let source_key: Vec<u8> = row.try_get("source_key")?;
        let authority_x: Vec<u8> = row.try_get("source_authority")?;
        let codec_x: Vec<u8> = row.try_get("relations")?;
        let other = SINK.replace("000000000001", "000000000003");
        let sink_y = if change_payload { other.as_str() } else { SINK };
        let input_y = if change_payload { 1 } else { 2 };
        store
            .commit_page(
                page_with_input(
                    &store,
                    &catalog,
                    "metadata",
                    "source-y",
                    2,
                    (Some(sink_y), input(input_y)?),
                )
                .await?,
            )
            .await?;
        let row =
            sqlx::query("SELECT source_key,source_authority FROM resource_facets WHERE id=?1")
                .bind(LINK)
                .fetch_one(&sql)
                .await?;
        assert_eq!(
            row.try_get::<Vec<u8>, _>("source_key")?,
            source_key,
            "root ownership intentionally omits input; grant authority must not"
        );
        assert_ne!(
            row.try_get::<Vec<u8>, _>("source_authority")?,
            authority_x,
            "both actual input and full source payload bind the grant"
        );
        let record_y = frozen(&sql, &catalog).await?.1;
        // Preserve Y's actual SQL row authority and record. Only replay X's old
        // dependency codec: no fabricated matching source hash or grant is added.
        sqlx::query("UPDATE resource_facets SET relations=?1 WHERE id=?2 AND source_key=?3")
            .bind(&codec_x)
            .bind(LINK)
            .bind(&source_key)
            .execute(&sql)
            .await?;
        // This independent, compiler-checked native ListLinks metadata branch
        // owns only Arn, so Y still owns SinkArn and the replayed dependency is
        // actually selected. It forces re-aggregation without replacing Y.
        store
            .commit_page(
                page_with_sink(&store, &catalog, "refresh", "reaggregate-y", 3, true, None).await?,
            )
            .await?;
        assert_eq!(frozen(&sql, &catalog).await?.1, record_y);
        assert_eq!(
            sqlx::query_scalar::<_, String>(
                "SELECT target_value FROM unresolved_refs WHERE from_id=?1"
            )
            .bind(LINK)
            .fetch_one(&sql)
            .await?,
            SINK
        );
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence WHERE from_id=?1 AND authority IS NOT NULL").bind(LINK).fetch_one(&sql).await?, 0);
        persist_oam(&store, &catalog, "ListSinks", "210987654321", 4).await?;
        assert!(
            store.projection_edges(None, 100).await?.edges.is_empty(),
            "X's retained grant cannot authorize Y's copied codec"
        );
        // An actual Y observation can issue Y's own proof and resolve only its
        // exact foreign target; this also proves the rejection is not a blanket ban.
        store
            .commit_page(
                page_with_input(
                    &store,
                    &catalog,
                    "metadata",
                    "fresh-y",
                    5,
                    (Some(sink_y), input(input_y)?),
                )
                .await?,
            )
            .await?;
        store
            .commit_page(
                oam_page_in_scope(
                    &store,
                    &catalog,
                    "ListSinks",
                    "210987654321",
                    6,
                    ("us-east-1", LINK, sink_y),
                )
                .await?,
            )
            .await?;
        let edges = store.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1);
        let edge = edges.first().ok_or("fresh Y edge absent")?;
        assert_eq!(
            (
                edge.from.id.as_str(),
                edge.to.account.as_str(),
                edge.to.id.as_str(),
                edge.relation.as_str()
            ),
            (LINK, "210987654321", sink_y, "TARGETS_SINK")
        );
        sql.close().await;
    }
    Ok(())
}
