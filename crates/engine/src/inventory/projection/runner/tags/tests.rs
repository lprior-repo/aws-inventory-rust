use super::*;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn key(account: &str, id: &str) -> Result<ResourceKey, Box<dyn std::error::Error>> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::EC2::Vpc")?,
        id: ResourceId::parse(id)?,
    })
}

#[test]
fn scoped_tag_pairs_are_shared_without_resource_cliques() -> TestResult {
    let resource = key("123456789012", "vpc-a")?;
    let other = key("123456789012", "vpc-b")?;
    let pair = crate::inventory::ProjectionTag {
        key: "team".into(),
        value: "platform".into(),
    };
    assert_eq!(
        tag_identity(&resource, &pair)?,
        tag_identity(&other, &pair)?
    );
    assert_ne!(
        tag_identity(&resource, &pair)?,
        tag_identity(&key("000000000000", "vpc-a")?, &pair)?
    );
    let mut other_region = resource.clone();
    other_region.region = Region::parse("us-west-2")?;
    assert_ne!(
        tag_identity(&resource, &pair)?,
        tag_identity(&other_region, &pair)?
    );
    Ok(())
}

#[test]
fn tag_identity_preserves_pair_boundaries_and_key_value_order() -> TestResult {
    let resource = key("123456789012", "vpc-a")?;
    let pair = |key: &str, value: &str| crate::inventory::ProjectionTag {
        key: key.into(),
        value: value.into(),
    };
    let identity = tag_identity(&resource, &pair("ab", "c"))?;
    assert_ne!(identity, tag_identity(&resource, &pair("a", "bc"))?);
    assert_ne!(identity, tag_identity(&resource, &pair("c", "ab"))?);
    Ok(())
}
