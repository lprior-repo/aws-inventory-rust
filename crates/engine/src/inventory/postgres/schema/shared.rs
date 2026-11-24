use crate::inventory::StoreError;
use futures_util::{TryStreamExt, stream};
use sqlx::{Postgres, Transaction};

const TABLES: &[&str] = &[
    "CREATE SEQUENCE inventory.projection_revision_seq AS bigint MINVALUE 1 NO CYCLE",
    "CREATE SEQUENCE inventory.projection_delivery_generation_seq AS bigint MINVALUE 1 NO CYCLE",
    "CREATE TABLE inventory.metadata(key text PRIMARY KEY,value bytea NOT NULL)",
    "CREATE TABLE inventory.kinds(kind text PRIMARY KEY,table_name text NOT NULL UNIQUE)",
    "CREATE TABLE inventory.kind_schemas(kind text PRIMARY KEY,table_name text NOT NULL,definition bytea NOT NULL)",
    "CREATE TABLE inventory.resources(account text NOT NULL,region text NOT NULL,kind text NOT NULL,id text NOT NULL,name text,content_hash bytea NOT NULL,first_seen bigint NOT NULL,last_changed bigint NOT NULL,PRIMARY KEY(account,region,kind,id))",
    "CREATE TABLE inventory.resource_facets(account text NOT NULL,region text NOT NULL,kind text NOT NULL,id text NOT NULL,service text NOT NULL,operation text NOT NULL,source_key bytea NOT NULL,source_authority bytea CHECK(source_authority IS NULL OR octet_length(source_authority)=32),bound_request bytea,precedence bigint NOT NULL CHECK(precedence BETWEEN 0 AND 65535),record bytea NOT NULL,owned bytea NOT NULL,exclusions bytea NOT NULL,relations bytea NOT NULL,name text,name_owned bigint NOT NULL CHECK(name_owned IN(0,1)),PRIMARY KEY(account,region,kind,id,service,operation,source_key),FOREIGN KEY(account,region,kind,id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.sweeps(sweep_id text PRIMARY KEY,account text NOT NULL,partition text NOT NULL DEFAULT 'aws',started_at bigint NOT NULL,ended_at bigint NOT NULL DEFAULT 0,completed bigint NOT NULL DEFAULT 0)",
    "CREATE TABLE inventory.resource_scopes(account text NOT NULL,region text NOT NULL,kind text NOT NULL,id text NOT NULL,scope_key bytea NOT NULL,last_seen bigint NOT NULL,missing_complete bigint NOT NULL DEFAULT 0,observed_sweep text NOT NULL,completed_sweep text NOT NULL DEFAULT '',PRIMARY KEY(account,region,kind,id,scope_key),FOREIGN KEY(account,region,kind,id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.tags(account text NOT NULL,region text NOT NULL,kind text NOT NULL,id text NOT NULL,tag_key text NOT NULL,tag_value text NOT NULL,PRIMARY KEY(account,region,kind,id,tag_key),FOREIGN KEY(account,region,kind,id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.tag_observations(account text NOT NULL,region text NOT NULL,kind text NOT NULL,id text NOT NULL,coverage text NOT NULL CHECK(coverage IN('complete','partial','failed','unobserved')),observed_at bigint NOT NULL CHECK(observed_at>=0),last_complete_at bigint CHECK(last_complete_at>=0),PRIMARY KEY(account,region,kind,id),FOREIGN KEY(account,region,kind,id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.edges(from_account text NOT NULL,from_region text NOT NULL,from_kind text NOT NULL,from_id text NOT NULL,to_account text NOT NULL,to_region text NOT NULL,to_kind text NOT NULL,to_id text NOT NULL,relation text NOT NULL,source text NOT NULL,PRIMARY KEY(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(to_account,to_region,to_kind,to_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.unresolved_refs(from_account text NOT NULL,from_region text NOT NULL,from_kind text NOT NULL,from_id text NOT NULL,target_scope bytea NOT NULL,target_kind text NOT NULL,target_path bytea NOT NULL,target_value text NOT NULL,relation text NOT NULL,source text NOT NULL,target_kinds bytea NOT NULL DEFAULT ''::bytea,resolution text NOT NULL DEFAULT 'unresolved' CHECK(resolution IN('unresolved','resolved','ambiguous')),PRIMARY KEY(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.resource_parents(parent_account text NOT NULL,parent_region text NOT NULL,parent_kind text NOT NULL,parent_id text NOT NULL,child_account text NOT NULL,child_region text NOT NULL,child_kind text NOT NULL,child_id text NOT NULL,scope_key bytea NOT NULL,PRIMARY KEY(parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key),FOREIGN KEY(parent_account,parent_region,parent_kind,parent_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(child_account,child_region,child_kind,child_id) REFERENCES inventory.resources(account,region,kind,id) ON DELETE CASCADE)",
    "CREATE TABLE inventory.scope_outcomes(sweep_id text NOT NULL,scope_key bytea NOT NULL,outcome text NOT NULL,observed_at bigint NOT NULL,detail text CHECK(detail IS NULL OR octet_length(detail)<=1024),PRIMARY KEY(sweep_id,scope_key))",
    "CREATE TABLE inventory.work_queue(sweep_id text NOT NULL,scope_key bytea NOT NULL,token bytea,bindings bytea NOT NULL,context bytea NOT NULL DEFAULT ''::bytea,state bigint NOT NULL,generation bigint NOT NULL,authority_generation bigint NOT NULL DEFAULT 0 CHECK(authority_generation>=0),authority_complete bigint NOT NULL DEFAULT 0 CHECK(authority_complete IN(0,1)),last_complete bigint NOT NULL DEFAULT -1 CHECK(last_complete>=-1),lease_until bigint NOT NULL,PRIMARY KEY(sweep_id,scope_key))",
    "CREATE TABLE inventory.work_tokens(sweep_id text NOT NULL,scope_key bytea NOT NULL,digest bytea NOT NULL CHECK(octet_length(digest)=32),PRIMARY KEY(sweep_id,scope_key,digest),FOREIGN KEY(sweep_id,scope_key) REFERENCES inventory.work_queue(sweep_id,scope_key) ON DELETE CASCADE)",
    "CREATE TABLE inventory.catalog_operations(service text NOT NULL,operation text NOT NULL,PRIMARY KEY(service,operation))",
    "CREATE TABLE inventory.projection_outbox(resource_key bytea PRIMARY KEY,revision bigint NOT NULL,deleted boolean NOT NULL,full_rebuild boolean NOT NULL DEFAULT false)",
    "CREATE TABLE inventory.learned_rates(service text NOT NULL,operation text NOT NULL,rate double precision NOT NULL,sample_count bigint NOT NULL,updated_at bigint NOT NULL,PRIMARY KEY(service,operation))",
    "CREATE INDEX work_queue_ready ON inventory.work_queue(sweep_id,state,last_complete,scope_key)",
    "CREATE INDEX scope_outcomes_complete ON inventory.scope_outcomes(scope_key,outcome,observed_at)",
    "CREATE INDEX unresolved_refs_target ON inventory.unresolved_refs(target_scope,target_kind)",
    "CREATE INDEX resource_scopes_key ON inventory.resource_scopes(scope_key)",
];

pub(super) async fn create(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    stream::iter(TABLES.iter().copied().map(Ok))
        .try_fold(tx, |tx, statement| async move {
            sqlx::query(statement).execute(&mut **tx).await?;
            Ok::<_, StoreError>(tx)
        })
        .await?;
    Ok(())
}
