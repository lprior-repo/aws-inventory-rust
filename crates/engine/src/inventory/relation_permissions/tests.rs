use super::*;
use inventory_core::inventory::{AccountId, Relation, ResourceId};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source(region: Region) -> TestResult<ResourceKey> {
    Ok(ResourceKey { account: AccountId::parse("123456789012")?, region,
        kind: Kind::parse("AWS::Oam::Link")?, id: ResourceId::parse("arn:aws:oam:us-east-1:123456789012:link/source")? })
}

#[test]
fn given_execution_account_rule_when_arn_claims_foreign_owner_region_partition_or_wildcard_then_it_is_not_authority() -> TestResult {
    let source = source(Region::parse("us-east-1")?)?;
    assert!(execution_arn_matches(&source, &source.region, "arn:aws:oam:us-east-1:123456789012:sink/target"));
    for forged in ["arn:aws:oam:us-east-1:210987654321:sink/target", "arn:aws:oam:us-west-2:123456789012:sink/target",
        "arn:aws-cn:oam:us-east-1:123456789012:sink/target", "arn:aws:oam:us-east-1:123456789012:sink/*",
        "arn:aws:oam:us-east-1:123456789012:sink/targe?", "arn:aws:oam:us-east-1:123456789012:"] {
        assert!(!execution_arn_matches(&source, &source.region, forged), "{forged}");
    }
    Ok(())
}

#[test]
fn given_regional_source_when_arn_names_no_region_then_it_is_owned_but_other_regions_and_global_claims_stay_forged() -> TestResult {
    let regional = source(Region::parse("us-east-1")?)?;
    assert!(execution_arn_matches(&regional, &regional.region, "arn:aws:oam::123456789012:sink/target"));
    assert!(!execution_arn_matches(&regional, &regional.region, "arn:aws:oam:us-west-2:123456789012:sink/target"));
    let global = source(Region::Global)?;
    assert!(!execution_arn_matches(&global, &global.region, "arn:aws:oam:us-east-1:123456789012:sink/target"));
    Ok(())
}

#[test]
fn given_global_source_without_execution_home_when_home_region_is_claimed_then_no_region_is_invented() -> TestResult {
    let source = source(Region::Global)?;
    let target_kind = Kind::parse("AWS::Oam::Sink")?;
    let rule = RelationshipPlan { source_kind: source.kind.clone(), source: RelationSource::Request,
        source_path: vec![FieldId::new(1)?].into_boxed_slice(), target_kind: target_kind.clone(), target_kinds: vec![target_kind].into_boxed_slice(),
        target_path: vec![FieldId::new(1)?].into_boxed_slice(), relation: Relation::parse("TARGETS_SINK")?, scope: RelationScope::HomeRegion,
        owner: ReferenceOwnerRule::ExecutionAccount,
        source_path_mode: inventory_core::catalog::runtime::RelationPathMode::Fields,
        target_mode: inventory_core::catalog::runtime::RelationTargetMode::SelectedFields,
        guard: None, components: Box::new([]) };
    assert!(!owner_matches(&rule, &source, &source.account, &Region::parse("us-east-1")?, "target"));
    assert!(!owner_matches(&rule, &source, &source.account, &Region::parse("us-west-2")?, "target"));
    Ok(())
}

#[test]
fn given_global_source_when_arn_region_rule_names_regional_arn_then_only_same_account_regional_arns_are_authority() -> TestResult {
    let source = source(Region::Global)?;
    let target_kind = Kind::parse("AWS::KMS::Key")?;
    let rule = RelationshipPlan { source_kind: source.kind.clone(), source: RelationSource::Request,
        source_path: vec![FieldId::new(1)?].into_boxed_slice(), target_kind: target_kind.clone(), target_kinds: vec![target_kind].into_boxed_slice(),
        target_path: vec![FieldId::new(1)?].into_boxed_slice(), relation: Relation::parse("USES_KEY")?, scope: RelationScope::ArnRegion,
        owner: ReferenceOwnerRule::ExecutionAccount,
        source_path_mode: inventory_core::catalog::runtime::RelationPathMode::Fields,
        target_mode: inventory_core::catalog::runtime::RelationTargetMode::SelectedFields,
        guard: None, components: Box::new([]) };
    let key = "arn:aws:kms:us-east-1:123456789012:key/11111111-2222-3333-4444-555555555555";
    let us_east = Region::parse("us-east-1")?;
    assert!(owner_matches(&rule, &source, &source.account, &us_east, key));
    assert!(!owner_matches(&rule, &source, &source.account, &Region::Global, key));
    assert!(!owner_matches(&rule, &source, &AccountId::parse("210987654321")?, &us_east, key));
    assert!(!owner_matches(&rule, &source, &source.account, &Region::parse("us-west-2")?, key));
    assert!(!owner_matches(&rule, &source, &source.account, &us_east, "11111111-2222-3333-4444-555555555555"));
    Ok(())
}
