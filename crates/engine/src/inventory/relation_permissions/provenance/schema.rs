use super::StoreError;
use sqlx::{Row, Sqlite, Transaction};

pub(in crate::inventory) const SCHEMA: [&str; 2] = [
    "CREATE TABLE IF NOT EXISTS reviewed_reference_grants(proof BLOB NOT NULL CHECK(length(proof)=32),target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0),authority BLOB CHECK(authority IS NULL OR length(authority)=32),source_facet BLOB,source_authority BLOB CHECK(source_authority IS NULL OR length(source_authority)=32))",
    "CREATE TABLE IF NOT EXISTS reviewed_reference_evidence(proof BLOB NOT NULL CHECK(length(proof)=32),from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0),authority BLOB CHECK(authority IS NULL OR length(authority)=32),source_facet BLOB,source_authority BLOB CHECK(source_authority IS NULL OR length(source_authority)=32))",
];

pub(in crate::inventory) async fn initialize(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), StoreError> {
    for statement in SCHEMA {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    for (pragma, add_authority, add_facet, add_source_authority) in [
        (
            "PRAGMA table_info(reviewed_reference_grants)",
            "ALTER TABLE reviewed_reference_grants ADD COLUMN authority BLOB CHECK(authority IS NULL OR length(authority)=32)",
            "ALTER TABLE reviewed_reference_grants ADD COLUMN source_facet BLOB",
            "ALTER TABLE reviewed_reference_grants ADD COLUMN source_authority BLOB CHECK(source_authority IS NULL OR length(source_authority)=32)",
        ),
        (
            "PRAGMA table_info(reviewed_reference_evidence)",
            "ALTER TABLE reviewed_reference_evidence ADD COLUMN authority BLOB CHECK(authority IS NULL OR length(authority)=32)",
            "ALTER TABLE reviewed_reference_evidence ADD COLUMN source_facet BLOB",
            "ALTER TABLE reviewed_reference_evidence ADD COLUMN source_authority BLOB CHECK(source_authority IS NULL OR length(source_authority)=32)",
        ),
    ] {
        let columns = sqlx::query(pragma).fetch_all(&mut **tx).await?;
        let names = columns
            .iter()
            .map(|row| row.try_get::<String, _>("name"))
            .collect::<Result<Vec<_>, _>>()?;
        if !names.iter().any(|name| name == "authority") {
            sqlx::query(add_authority).execute(&mut **tx).await?;
        }
        if !names.iter().any(|name| name == "source_facet") {
            sqlx::query(add_facet).execute(&mut **tx).await?;
        }
        if !names.iter().any(|name| name == "source_authority") {
            sqlx::query(add_source_authority).execute(&mut **tx).await?;
        }
    }
    for statement in [
        "DROP INDEX IF EXISTS reviewed_reference_grant_unconstrained",
        "DROP INDEX IF EXISTS reviewed_reference_grant_target",
        "DROP INDEX IF EXISTS reviewed_reference_unconstrained",
        "DROP INDEX IF EXISTS reviewed_reference_target",
        "CREATE UNIQUE INDEX IF NOT EXISTS reviewed_reference_grant_unconstrained_v3 ON reviewed_reference_grants(proof,authority,source_facet,source_authority) WHERE target_id IS NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
        "CREATE UNIQUE INDEX IF NOT EXISTS reviewed_reference_grant_target_v3 ON reviewed_reference_grants(proof,authority,source_facet,source_authority,target_id) WHERE target_id IS NOT NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS reviewed_reference_grant_proof ON reviewed_reference_grants(proof)",
        "CREATE UNIQUE INDEX IF NOT EXISTS reviewed_reference_unconstrained_v3 ON reviewed_reference_evidence(proof,authority,source_facet,source_authority) WHERE target_id IS NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
        "CREATE UNIQUE INDEX IF NOT EXISTS reviewed_reference_target_v3 ON reviewed_reference_evidence(proof,authority,source_facet,source_authority,target_id) WHERE target_id IS NOT NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS reviewed_reference_proof ON reviewed_reference_evidence(proof)",
        "CREATE INDEX IF NOT EXISTS reviewed_reference_source ON reviewed_reference_evidence(from_account,from_region,from_kind,from_id)",
    ] {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    Ok(())
}

pub(in crate::inventory) async fn invalidate_prior_authority(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), StoreError> {
    for statement in [
        "CREATE TABLE IF NOT EXISTS archived_reviewed_reference_grants_pre_v3 AS SELECT * FROM reviewed_reference_grants WHERE 0",
        "CREATE TABLE IF NOT EXISTS archived_reviewed_reference_evidence_pre_v3 AS SELECT * FROM reviewed_reference_evidence WHERE 0",
        "INSERT INTO archived_reviewed_reference_grants_pre_v3 SELECT * FROM reviewed_reference_grants",
        "INSERT INTO archived_reviewed_reference_evidence_pre_v3 SELECT * FROM reviewed_reference_evidence",
        "UPDATE reviewed_reference_grants SET authority=NULL,source_facet=NULL,source_authority=NULL",
        "UPDATE reviewed_reference_evidence SET authority=NULL,source_facet=NULL,source_authority=NULL",
    ] {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    Ok(())
}
