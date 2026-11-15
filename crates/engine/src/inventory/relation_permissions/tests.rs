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
fn given_global_source_without_execution_home_when_home_region_is_claimed_then_no_region_is_invented() -> TestResult {
    let source = source(Region::Global)?;
    let target_kind = Kind::parse("AWS::Oam::Sink")?;
    let rule = RelationshipPlan { source_kind: source.kind.clone(), source: RelationSource::Request,
        source_path: vec![FieldId::new(1)?].into_boxed_slice(), target_kind: target_kind.clone(), target_kinds: vec![target_kind].into_boxed_slice(),
        target_path: vec![FieldId::new(1)?].into_boxed_slice(), relation: Relation::parse("TARGETS_SINK")?, scope: RelationScope::HomeRegion,
        owner: ReferenceOwnerRule::ExecutionAccount };
    assert!(!owner_matches(&rule, &source, &source.account, &Region::parse("us-east-1")?, "target"));
    assert!(!owner_matches(&rule, &source, &source.account, &Region::parse("us-west-2")?, "target"));
    Ok(())
}
