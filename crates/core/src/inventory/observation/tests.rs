use super::*;
use crate::{
    inventory::{AccountId, Kind, Region, ResourceId},
    record::{FieldId, Key, MapKeyExclusion, Value},
};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_shared_exclusions_when_observations_are_created_then_backing_paths_are_not_copied()
-> TestResult {
    // Given nonempty field and map-key exclusions shared by two observations.
    let field = FieldId::new(1)?;
    let exclusions = Exclusions::new([vec![field]])?.with_map_keys(vec![MapKeyExclusion::new(
        vec![FieldId::new(2)?],
        "secret",
    )?])?;
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-west-2")?,
        kind: Kind::parse("AWS::Test::Resource")?,
        id: ResourceId::parse("one")?,
    };
    let record = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(field),
        Value::Text("excluded".to_owned()),
    )])))?;
    let expected_hash =
        Record::encode(&Value::Map(BTreeMap::new()))?.content_hash(&Exclusions::none())?;
    let unexcluded_hash = record.content_hash(&Exclusions::none())?;
    // When observations take ownership independently of the source exclusion handle.
    let first = Observation::new(key.clone(), record.clone(), &exclusions, None)?;
    let second = Observation::new(key, record, &exclusions, None)?;
    // Then each uses the source backing allocations, not a per-row deep copy.
    for observation in [&first, &second] {
        assert_eq!(
            observation.hash_exclusions().paths().as_ptr(),
            exclusions.paths().as_ptr()
        );
        assert_eq!(
            observation.hash_exclusions().map_keys().as_ptr(),
            exclusions.map_keys().as_ptr()
        );
        assert_eq!(observation.content_hash(), expected_hash);
        assert_ne!(observation.content_hash(), unexcluded_hash);
    }
    drop(exclusions);
    assert_eq!(
        first.hash_exclusions().paths(),
        &[vec![field].into_boxed_slice()]
    );
    assert_eq!(first.hash_exclusions(), second.hash_exclusions());
    Ok(())
}
