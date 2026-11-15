#[path = "inventory_store/support.rs"]
mod support;
use support::*;
use inventory_core::{catalog::runtime::ContextProjectionPlan, inventory::{DiscoveryContext, DiscoveryFrame, OperationId, ScopeKey, ServiceId}, record::{FieldId, Key, Record, Value}};
use inventory_engine::inventory::{BinaryStore, StoreError, SweepId, WorkSpec};
use std::collections::{BTreeMap, BTreeSet};

fn context(name: &str) -> TestResult<DiscoveryContext> {
    context_at(name,0)
}

fn context_at(name: &str, ordinal: u32) -> TestResult<DiscoveryContext> {
    let field = FieldId::new(1)?;
    let row = Record::encode(&Value::Map(BTreeMap::from([(Key::Field(field), Value::Text(name.to_owned()))])))?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan { selected: vec![vec![field].into_boxed_slice()].into_boxed_slice(), ..ContextProjectionPlan::default() };
    let frame = DiscoveryFrame::project(ServiceId::parse("example")?, OperationId::parse("ListThings")?,
        ordinal, &projection, &row, &empty, &empty)?.with_branch(Some("ACCOUNT"))?;
    Ok(DiscoveryContext::empty().push(frame)?)
}

fn scoped_work(sweep: &SweepId, branch: &str, ancestry: &DiscoveryContext) -> TestResult<WorkSpec> {
    let scope = scope("us-east-1")?.with_branch(branch.to_owned())?.with_context(ancestry.discriminator()?);
    let bindings = Record::encode(&Value::Map(BTreeMap::new()))?.into_bytes();
    Ok(WorkSpec::new(sweep.clone(), scope, None, bindings)?.with_context(ancestry.encode()?)?)
}

#[tokio::test]
async fn given_two_branches_when_pages_restart_then_each_keeps_its_context_cursor_and_completion() -> TestResult {
    // Given two semantic branches sharing service, operation, region and cursor digest.
    let (directory, store, _) = declared_branch_fixture(&["ACCOUNT", "RESOURCE"]).await?;
    let sweep = SweepId::new("branches".to_owned())?;
    let account = context("account-parent")?;
    let resource = context("resource-parent")?;
    store.seed_works(&[scoped_work(&sweep,"ACCOUNT",&account)?, scoped_work(&sweep,"RESOURCE",&resource)?]).await?;
    let leases = [store.claim(&sweep,100,100).await?.ok_or("first branch absent")?,
        store.claim(&sweep,100,100).await?.ok_or("second branch absent")?];
    for lease in leases {
        let branch = lease.scope().branch().ok_or("branch lost")?.to_owned();
        let mut checkpoint = page(lease,Vec::new(),false,Some(branch.into_bytes()),101);
        checkpoint.continuation_digest = Some([7;32]);
        assert!(!store.commit_page(checkpoint).await?.repeated_token);
    }
    // When the real SQLite store is closed and reopened after lease expiry.
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let mut branches = BTreeSet::new();
    for _ in 0..2 {
        let lease = store.claim(&sweep,1000,100).await?.ok_or("resumed branch absent")?;
        let branch = lease.scope().branch().ok_or("resumed branch lost")?.to_owned();
        let expected = if branch == "ACCOUNT" { &account } else { &resource };
        assert_eq!(DiscoveryContext::decode(lease.context())?,*expected);
        assert_eq!(lease.scope().context(),Some(expected.discriminator()?));
        assert_eq!(lease.token(),Some(branch.as_bytes()));
        store.commit_page(page(lease,Vec::new(),true,None,1001)).await?;
        branches.insert(branch);
    }
    // Then both independent scopes completed, and neither digest was mistaken for a repeat.
    assert_eq!(branches,BTreeSet::from(["ACCOUNT".to_owned(),"RESOURCE".to_owned()]));
    let summary = store.sweep_summary(&sweep).await?;
    assert_eq!(summary.complete_scopes,2);
    assert_eq!(summary.partial_scopes,0);
    Ok(())
}

async fn complete_branch(store: &BinaryStore, sweep: &str, branch: &str, rows: Vec<inventory_core::inventory::Observation>, at: i64) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    let scope = scope("us-east-1")?.with_branch(branch.to_owned())?;
    let bindings = Record::encode(&Value::Map(BTreeMap::new()))?.into_bytes();
    store.seed_work(WorkSpec::new(sweep.clone(),scope,None,bindings)?).await?;
    let lease = store.claim(&sweep,at,100).await?.ok_or("branch scope absent")?;
    store.commit_page(page(lease,rows,true,None,at+1)).await?;
    Ok(())
}

#[tokio::test]
async fn given_branch_owned_resources_when_other_branch_is_empty_twice_then_deletion_stays_isolated() -> TestResult {
    // Given independent resources observed by the two branches of one modeled operation.
    let (_directory,store,_) = declared_branch_fixture(&["ACCOUNT", "RESOURCE"]).await?;
    complete_branch(&store,"initial-account","ACCOUNT",vec![observation("account-policy","account")?],100).await?;
    complete_branch(&store,"initial-resource","RESOURCE",vec![observation("resource-policy","resource")?],200).await?;
    // When only RESOURCE has two complete empty observations.
    complete_branch(&store,"resource-miss-1","RESOURCE",Vec::new(),300).await?;
    assert!(store.projection_resource(&resource("resource-policy")?).await?.is_some());
    complete_branch(&store,"resource-miss-2","RESOURCE",Vec::new(),400).await?;
    // Then RESOURCE's missing resource is deleted without marking ACCOUNT missing.
    assert!(store.projection_resource(&resource("resource-policy")?).await?.is_none());
    let retained = store.projection_resource(&resource("account-policy")?).await?.ok_or("unrelated branch deleted")?;
    assert_eq!(retained.key.id.as_str(),"account-policy");
    Ok(())
}

#[tokio::test]
async fn given_context_without_matching_scope_digest_when_seeded_then_no_work_is_committed() -> TestResult {
    // Given a valid minimal ancestry paired with an unrelated trusted scope identity.
    let (_directory,store,_) = declared_branch_fixture(&["ACCOUNT", "RESOURCE"]).await?;
    let sweep = SweepId::new("context-mismatch".to_owned())?;
    let ancestry = context("real-parent")?;
    let invalid_scope = scope("us-east-1")?.with_branch("RESOURCE".to_owned())?;
    let invalid = WorkSpec::new(sweep.clone(),invalid_scope,None,Vec::new())?.with_context(ancestry.encode()?)?;
    // When the durable public API validates the work before opening its transaction.
    let result = store.seed_work(invalid).await;
    // Then the mismatch is explicit and no lease can be claimed.
    assert!(matches!(result,Err(StoreError::Invalid("work discovery context digest mismatch"))));
    assert!(store.claim(&sweep,100,100).await?.is_none());
    Ok(())
}

#[test]
fn given_invalid_branch_names_when_constructed_then_scope_identity_is_rejected() -> TestResult {
    for branch in [String::new(),"a".repeat(65),"two branches".to_owned(),"../escape".to_owned(),"é".to_owned()] {
        assert!(matches!(scope("us-east-1")?.with_branch(branch),Err(inventory_core::inventory::InventoryError::Branch)));
    }
    let maximum: ScopeKey = scope("us-east-1")?.with_branch("a".repeat(64))?;
    assert_eq!(maximum.branch().map(str::len),Some(64));
    Ok(())
}

async fn complete_context(store: &BinaryStore, sweep: &str, ancestry: &DiscoveryContext,
    rows: Vec<inventory_core::inventory::Observation>, at: i64) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(scoped_work(&sweep,"RESOURCE",ancestry)?).await?;
    let lease = store.claim(&sweep,at,100).await?.ok_or("contextual detail scope absent")?;
    store.commit_page(page(lease,rows,true,None,at+1)).await?;
    Ok(())
}

#[tokio::test]
async fn given_reordered_parent_context_when_detail_is_missing_twice_then_volatile_old_owner_cannot_block_deletion() -> TestResult {
    // Given the same semantic request observed before and after its parent moves in a list.
    let (directory,store,_) = declared_branch_fixture(&["ACCOUNT", "RESOURCE"]).await?;
    let before = context_at("stable-parent",0)?;
    let reordered = context_at("stable-parent",1)?;
    assert_ne!(before.discriminator()?,reordered.discriminator()?);
    complete_context(&store,"before-reorder",&before,vec![observation("detail","stable")?],100).await?;
    complete_context(&store,"after-reorder",&reordered,vec![observation("detail","stable")?],200).await?;
    // When the persisted current detail scope completes twice without that resource.
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    complete_context(&store,"detail-missing-once",&reordered,Vec::new(),300).await?;
    assert!(store.projection_resource(&resource("detail")?).await?.is_some());
    complete_context(&store,"detail-missing-twice",&reordered,Vec::new(),400).await?;
    // Then the old execution ancestry does not retain immortal resource ownership.
    assert!(store.projection_resource(&resource("detail")?).await?.is_none());
    Ok(())
}
