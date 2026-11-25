use super::*;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn owner() -> Result<ResourceKey, inventory_core::inventory::InventoryError> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::Example::Thing")?,
        id: ResourceId::parse("owner")?,
    })
}
fn page(
    owner: &ResourceKey,
    coverage: TagCoverage,
    pairs: &[(&str, &str)],
) -> Result<TagObservation, inventory_core::inventory::InventoryError> {
    TagObservation::new(
        owner.clone(),
        coverage,
        pairs
            .iter()
            .map(|(key, value)| Tag::new(*key, *value))
            .collect::<Result<Vec<_>, _>>()?,
    )
}
fn empty() -> Result<Record, inventory_core::record::RecordError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new()))
}

#[test]
fn given_multiple_pages_when_reopened_then_only_terminal_union_is_complete() -> TestResult {
    let owner = owner()?;
    let pending = Accumulation::restore(&owner, None)?.append(page(
        &owner,
        TagCoverage::Complete,
        &[("z", ""), ("a", "one")],
    )?)?;
    assert_eq!(pending.observation(false)?.coverage(), TagCoverage::Partial);
    let checkpoint = pending.checkpoint(empty()?)?;
    let restored = Accumulation::restore(&owner, Some(checkpoint.as_bytes()))?.append(page(
        &owner,
        TagCoverage::Complete,
        &[("m", "two"), ("a", "one")],
    )?)?;
    let final_snapshot = restored.observation(true)?;
    assert_eq!(final_snapshot.coverage(), TagCoverage::Complete);
    assert_eq!(final_snapshot.resource(), &owner);
    assert_eq!(
        final_snapshot
            .tags()
            .iter()
            .map(|tag| (tag.key(), tag.value()))
            .collect::<Vec<_>>(),
        vec![("a", "one"), ("m", "two"), ("z", "")]
    );
    Ok(())
}

#[test]
fn given_missing_page_when_later_pages_finish_then_no_authoritative_snapshot_is_emitted()
-> TestResult {
    let owner = owner()?;
    let missing =
        Accumulation::restore(&owner, None)?.append(page(&owner, TagCoverage::Unobserved, &[])?)?;
    let checkpoint = missing.checkpoint(empty()?)?;
    let restored = Accumulation::restore(&owner, Some(checkpoint.as_bytes()))?.append(page(
        &owner,
        TagCoverage::Complete,
        &[("visible", "pair")],
    )?)?;
    assert_eq!(restored.observation(true)?.coverage(), TagCoverage::Partial);
    assert_eq!(
        restored.observation(true)?.tags(),
        &[Tag::new("visible", "pair")?]
    );
    let empty_missing =
        Accumulation::restore(&owner, None)?.append(page(&owner, TagCoverage::Unobserved, &[])?)?;
    assert_eq!(
        empty_missing.observation(true)?.coverage(),
        TagCoverage::Unobserved
    );
    let explicit_empty =
        Accumulation::restore(&owner, None)?.append(page(&owner, TagCoverage::Complete, &[])?)?;
    assert_eq!(
        explicit_empty.observation(true)?.coverage(),
        TagCoverage::Complete
    );
    Ok(())
}

#[test]
fn given_owner_bound_checkpoint_when_any_owner_dimension_changes_then_restore_is_rejected()
-> TestResult {
    let owner = owner()?;
    let checkpoint = Accumulation::restore(&owner, None)?
        .append(page(&owner, TagCoverage::Complete, &[("known", "pair")])?)?
        .checkpoint(empty()?)?;
    let mut others = vec![owner.clone(); 4];
    others.get_mut(0).ok_or("owner absent")?.account = AccountId::parse("222222222222")?;
    others.get_mut(1).ok_or("owner absent")?.region = Region::Global;
    others.get_mut(2).ok_or("owner absent")?.kind = Kind::parse("AWS::Example::Other")?;
    others.get_mut(3).ok_or("owner absent")?.id = ResourceId::parse("different")?;
    for other in others {
        assert!(Accumulation::restore(&other, Some(checkpoint.as_bytes())).is_err());
    }
    assert!(Accumulation::restore(&owner, Some(empty()?.as_bytes())).is_err());
    Ok(())
}

#[test]
fn given_conflicts_or_excess_pairs_when_accumulating_then_no_complete_snapshot_is_available()
-> TestResult {
    let owner = owner()?;
    let pending = Accumulation::restore(&owner, None)?.append(page(
        &owner,
        TagCoverage::Complete,
        &[("same", "one")],
    )?)?;
    assert!(
        pending
            .append(page(&owner, TagCoverage::Complete, &[("same", "two")])?)
            .is_err()
    );
    let tags = (0..MAX_RESOURCE_TAGS)
        .map(|index| Tag::new(format!("key-{index:03}"), "value"))
        .collect::<Result<Vec<_>, _>>()?;
    let pending = Accumulation::restore(&owner, None)?.append(TagObservation::new(
        owner.clone(),
        TagCoverage::Complete,
        tags,
    )?)?;
    assert_eq!(pending.observation(true)?.tags().len(), MAX_RESOURCE_TAGS);
    assert!(
        pending
            .append(page(
                &owner,
                TagCoverage::Complete,
                &[("overflow", "value")]
            )?)
            .is_err()
    );
    Ok(())
}
