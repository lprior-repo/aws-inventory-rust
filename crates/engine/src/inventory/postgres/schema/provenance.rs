use crate::inventory::StoreError;
use sqlx::{Postgres, Transaction};

const STATEMENTS: &[&str] = &[
    "CREATE TABLE inventory.reviewed_reference_grants(proof bytea NOT NULL CHECK(octet_length(proof)=32),target_id text CHECK(target_id IS NULL OR length(target_id)>0),authority bytea CHECK(authority IS NULL OR octet_length(authority)=32),source_facet bytea,source_authority bytea CHECK(source_authority IS NULL OR octet_length(source_authority)=32))",
    "CREATE TABLE inventory.reviewed_reference_evidence(proof bytea NOT NULL CHECK(octet_length(proof)=32),from_account text NOT NULL,from_region text NOT NULL,from_kind text NOT NULL,from_id text NOT NULL,target_id text CHECK(target_id IS NULL OR length(target_id)>0),authority bytea CHECK(authority IS NULL OR octet_length(authority)=32),source_facet bytea,source_authority bytea CHECK(source_authority IS NULL OR octet_length(source_authority)=32),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE UNIQUE INDEX reviewed_reference_grant_unconstrained_v3 ON inventory.reviewed_reference_grants(proof,authority,source_facet,source_authority) WHERE target_id IS NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
    "CREATE UNIQUE INDEX reviewed_reference_grant_target_v3 ON inventory.reviewed_reference_grants(proof,authority,source_facet,source_authority,target_id) WHERE target_id IS NOT NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
    "CREATE INDEX reviewed_reference_grant_proof ON inventory.reviewed_reference_grants(proof)",
    "CREATE UNIQUE INDEX reviewed_reference_unconstrained_v3 ON inventory.reviewed_reference_evidence(proof,authority,source_facet,source_authority) WHERE target_id IS NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
    "CREATE UNIQUE INDEX reviewed_reference_target_v3 ON inventory.reviewed_reference_evidence(proof,authority,source_facet,source_authority,target_id) WHERE target_id IS NOT NULL AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL",
    "CREATE INDEX reviewed_reference_proof ON inventory.reviewed_reference_evidence(proof)",
    "CREATE INDEX reviewed_reference_source ON inventory.reviewed_reference_evidence(from_account,from_region,from_kind,from_id)",
];

pub(super) async fn create(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    for statement in STATEMENTS {
        sqlx::query(*statement).execute(&mut **tx).await?;
    }
    Ok(())
}
