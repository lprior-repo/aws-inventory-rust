use super::*;
use inventory_core::{
    inventory::ResourceId,
    record::{FieldId, Key},
};

fn operation_error<T>(result: Result<T, SweepError>, expected: &'static str) {
    assert!(matches!(result, Err(SweepError::Operation(message)) if message == expected));
}

#[tokio::test]
async fn given_fresh_work_without_checkpoint_when_scope_or_kind_changes_then_the_exact_owner_guard_rejects_it()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag plan missing")?;
    let (directory, store, owner, sweep) = seeded(&compiled, "scope-guards").await?;
    store
        .finish_scope(
            claim(&store, &sweep, 2).await?,
            ScopeOutcome::Failed,
            3,
            None,
        )
        .await?;
    let account_owner = {
        let mut key = owner.clone();
        key.account = AccountId::parse("222222222222")?;
        key
    };
    let region_owner = {
        let mut key = owner.clone();
        key.region = Region::parse("us-west-2")?;
        key
    };
    let kind_owner = {
        let mut key = owner.clone();
        key.kind = Kind::parse("AWS::IAM::User")?;
        key
    };
    let cases = [
        (
            None,
            owner.account.clone(),
            "tag endpoint requires trusted parent work",
        ),
        (
            Some(account_owner.clone()),
            account_owner.account,
            "tag endpoint owner scope differs from trusted parent",
        ),
        (
            Some(region_owner),
            owner.account.clone(),
            "tag endpoint owner scope differs from trusted parent",
        ),
        (
            Some(kind_owner),
            owner.account.clone(),
            "tag endpoint owner kind has no reviewed parent source",
        ),
    ];
    for (index, (parent, account, expected)) in cases.into_iter().enumerate() {
        let attempt = SweepId::new(format!("scope-guard-{index}"))?;
        let scope = ScopeKey::new(
            account,
            owner.region.clone(),
            ServiceId::parse("iam")?,
            OperationId::parse("ListRoleTags")?,
            parent,
        )?;
        store
            .seed_work(WorkSpec::new(
                attempt.clone(),
                scope,
                None,
                empty()?.into_bytes(),
            )?)
            .await?;
        let lease = claim(&store, &attempt, 4).await?;
        assert!(
            lease.token().is_none(),
            "scope guard must not be masked by accumulator digest validation"
        );
        operation_error(
            prepare_page(
                &catalog,
                &owner.account,
                10,
                &lease,
                plan,
                decode_cursor(None)?,
                xml(plan, Some(&[("wrong", "pair")]), None)?,
            ),
            expected,
        );
        store
            .finish_scope(lease, ScopeOutcome::Failed, 5, None)
            .await?;
        assert_eq!(
            tags(&store, &owner).await?,
            vec![("known".into(), "old".into())]
        );
    }
    let mut alternate = owner.clone();
    alternate.id = ResourceId::parse("another-trusted-role")?;
    let attempt = SweepId::new("scope-valid-alternate".to_owned())?;
    let scope = ScopeKey::new(
        owner.account.clone(),
        owner.region.clone(),
        ServiceId::parse("iam")?,
        OperationId::parse("ListRoleTags")?,
        Some(alternate.clone()),
    )?;
    store
        .seed_work(WorkSpec::new(
            attempt.clone(),
            scope,
            None,
            empty()?.into_bytes(),
        )?)
        .await?;
    let lease = claim(&store, &attempt, 6).await?;
    let accepted = prepare_page(
        &catalog,
        &owner.account,
        10,
        &lease,
        plan,
        decode_cursor(None)?,
        xml(plan, Some(&[("valid", "pair")]), None)?,
    )?;
    assert_eq!(pair(&accepted.tags)?.resource(), &alternate);
    assert_eq!(pair(&accepted.tags)?.coverage(), TagCoverage::Complete);
    store
        .finish_scope(lease, ScopeOutcome::Complete, 7, None)
        .await?;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_tampered_pending_bytes_when_reseeded_and_reopened_then_each_restore_guard_prevents_authoritative_tags()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag plan missing")?;
    let (directory, store, owner, sweep) = seeded(&compiled, "tamper-source").await?;
    let lease = claim(&store, &sweep, 2).await?;
    let parts = consume(
        &catalog,
        &lease,
        xml(plan, Some(&[("pending", "pair")]), Some("continue"))?,
        10,
    )?;
    let checkpoint = parts
        .next
        .as_ref()
        .ok_or("checkpoint missing")?
        .cursor
        .clone();
    store
        .finish_scope(lease, ScopeOutcome::Failed, 3, None)
        .await?;
    let mutations = [
        ("digest", "tag checkpoint owner differs from trusted parent"),
        ("coverage", "tag checkpoint has invalid coverage"),
        ("duplicates", "tag checkpoint contains duplicate keys"),
        ("owner", "tag checkpoint owner differs from trusted parent"),
    ];
    let mut store = store;
    for (mutation, expected) in mutations {
        let Value::Map(mut cursor) = ValueRef::Map(checkpoint.root()).to_value()? else {
            return Err("cursor not a map".into());
        };
        let Some(Value::Map(pending)) = cursor.get_mut(&Key::Field(FieldId::new(65_529)?)) else {
            return Err("pending map missing".into());
        };
        let mut trusted = owner.clone();
        match mutation {
            "digest" => {
                pending.insert(Key::Field(FieldId::new(1)?), Value::Bytes(vec![0; 32]));
            }
            "coverage" => {
                pending.insert(Key::Field(FieldId::new(3)?), Value::Text("false".into()));
            }
            "duplicates" => {
                pending.insert(
                    Key::Field(FieldId::new(2)?),
                    Value::List(vec![
                        Value::List(vec![Value::Text("same".into()), Value::Text("one".into())]),
                        Value::List(vec![Value::Text("same".into()), Value::Text("two".into())]),
                    ]),
                );
            }
            "owner" => {
                trusted.id = ResourceId::parse("different-trusted-owner")?;
            }
            _ => return Err("unknown mutation".into()),
        }
        let token = Record::encode(&Value::Map(cursor))?;
        let attempt = SweepId::new(format!("tamper-{mutation}"))?;
        let scope = ScopeKey::new(
            owner.account.clone(),
            owner.region.clone(),
            ServiceId::parse("iam")?,
            OperationId::parse("ListRoleTags")?,
            Some(trusted),
        )?;
        store
            .seed_work(WorkSpec::new(
                attempt.clone(),
                scope,
                Some(token.into_bytes()),
                empty()?.into_bytes(),
            )?)
            .await?;
        drop(store);
        store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        let lease = claim(&store, &attempt, 4).await?;
        operation_error(
            prepare_page(
                &catalog,
                &owner.account,
                10,
                &lease,
                plan,
                decode_cursor(lease.token())?,
                xml(plan, Some(&[("wrong", "pair")]), None)?,
            ),
            expected,
        );
        store
            .finish_scope(lease, ScopeOutcome::Failed, 5, None)
            .await?;
        assert_eq!(
            tags(&store, &owner).await?,
            vec![("known".into(), "old".into())]
        );
    }
    drop(store);
    directory.close()?;
    Ok(())
}
