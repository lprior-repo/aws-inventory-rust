use super::*;
pub(in super::super) fn historical_evidence(
    catalog: &RuntimeCatalog<'_>,
    reference: &UnresolvedReference,
) -> TestResult<([u8; 32], ResourceKey)> {
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut hash = Sha256::new();
    // This is the historical source's literal domain, despite its V2 facet marker.
    hash.update(b"aws-inventory-reviewed-reference-v1");
    hash.update(registry.digest());
    for text in [
        reference.from.account.as_str(),
        reference.from.region.as_str(),
        reference.from.kind.as_str(),
        reference.from.id.as_str(),
        reference.target.account().as_str(),
        reference.target.region().as_str(),
        reference.target.kind().as_str(),
        reference.target.value().as_str(),
        reference.relation.as_str(),
        reference.source.as_str(),
    ] {
        hash.update(text.len().to_le_bytes());
        hash.update(text.as_bytes());
    }
    hash.update(reference.target.path().len().to_le_bytes());
    for field in reference.target.path() {
        hash.update(field.value().to_le_bytes());
    }
    hash.update(reference.target.candidate_kinds().len().to_le_bytes());
    for kind in reference.target.candidate_kinds() {
        hash.update(kind.as_str().len().to_le_bytes());
        hash.update(kind.as_str().as_bytes());
    }
    Ok((hash.finalize().into(), reference.from.clone()))
}

pub(in super::super) async fn old_image(
    directory: &tempfile::TempDir,
    (proof, from): &([u8; 32], ResourceKey),
) -> TestResult {
    let pool = pool(directory).await?;
    let mut tx = pool.begin().await?;
    let source: Vec<u8> =
        sqlx::query_scalar("SELECT scope_key FROM work_queue WHERE sweep_id='old-discover'")
            .fetch_one(&mut *tx)
            .await?;
    let bytes: Vec<u8> = sqlx::query_scalar("SELECT relations FROM resource_facets WHERE id=?1")
        .bind(LINK)
        .fetch_one(&mut *tx)
        .await?;
    let relations = Record::from_bytes(bytes)?;
    let Value::List(mut entries) = relations
        .root()
        .get_field(FieldId::new(2)?)?
        .ok_or("facet references absent")?
        .to_value()?
    else {
        return Err("facet references are not a list".into());
    };
    assert_eq!(entries.len(), 1);
    for entry in &mut entries {
        let Value::List(tuple) = entry else {
            return Err("facet reference tuple absent".into());
        };
        assert_eq!(tuple.len(), 8);
        tuple.truncate(7); // The actual pre-V3 codec ended at the strict target pin.
    }
    let old_relations = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::List(entries),
    )])))?;
    sqlx::query("UPDATE resource_facets SET source_key=?1,relations=?2 WHERE id=?3")
        .bind(&source)
        .bind(old_relations.as_bytes())
        .bind(LINK)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE resource_scopes SET scope_key=?1,missing_complete=1,observed_sweep='old-discover',completed_sweep='old-discover' WHERE id=?2").bind(&source).bind(LINK).execute(&mut *tx).await?;
    for statement in [
        "DROP TABLE reviewed_reference_evidence",
        "DROP TABLE reviewed_reference_grants",
        "CREATE TABLE reviewed_reference_grants(proof BLOB NOT NULL CHECK(length(proof)=32),target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "CREATE TABLE reviewed_reference_evidence(proof BLOB NOT NULL CHECK(length(proof)=32),from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "ALTER TABLE resource_facets DROP COLUMN source_authority",
        "ALTER TABLE work_queue DROP COLUMN authority_generation",
        "ALTER TABLE work_queue DROP COLUMN authority_complete",
        "DROP TABLE legacy_observations",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_misses",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_completed_sweep",
        "DELETE FROM metadata WHERE key='canonical_facet_dependencies_v3'",
        "INSERT INTO metadata(key,value) VALUES('canonical_facet_dependencies_v2',X'01') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    ] {
        sqlx::query(statement).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO reviewed_reference_grants(proof,target_id) VALUES(?1,NULL)")
        .bind(proof.as_slice())
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO reviewed_reference_evidence(proof,from_account,from_region,from_kind,from_id,target_id) VALUES(?1,?2,?3,?4,?5,NULL)")
        .bind(proof.as_slice()).bind(from.account.as_str()).bind(from.region.as_str()).bind(from.kind.as_str()).bind(from.id.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    pool.close().await;
    Ok(())
}
