use super::encode_dependencies;
use crate::inventory::reference_candidates::stored_path;
use crate::inventory::relation_permissions::{SourceAuthority, reference_dependencies};
use crate::inventory::{SchemaRegistry, StoreError, WorkLease};
use inventory_core::inventory::{
    Observation, ReferenceTarget, ResourceId, ResourceKey, UnresolvedReference,
};
use inventory_core::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

struct Encoder<'a> {
    registry: &'a SchemaRegistry,
    lease: &'a WorkLease,
    observation: &'a Observation,
    authority: Option<&'a SourceAuthority<'a, 'a>>,
}

pub(in crate::inventory) fn encode<'a>(
    registry: &SchemaRegistry,
    lease: &WorkLease,
    observation: &Observation,
    references: impl Iterator<Item = (&'a UnresolvedReference, Option<&'a ResourceId>)>,
    authority: Option<&SourceAuthority<'_, '_>>,
) -> Result<Record, StoreError> {
    let encoder = Encoder {
        registry,
        lease,
        observation,
        authority,
    };
    let values = references
        .map(|(reference, pin)| encoder.reference(reference, pin))
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::List(values),
    )])))?)
}

impl Encoder<'_> {
    fn reference(
        &self,
        reference: &UnresolvedReference,
        pin: Option<&ResourceId>,
    ) -> Result<Value, StoreError> {
        let authority = self.authority(reference)?;
        let dependencies =
            reference_dependencies(self.registry, self.lease, self.observation, reference)?;
        Ok(Value::List(vec![
            Value::Bytes(target_key(&reference.target)?),
            Value::Text(reference.relation.as_str().to_owned()),
            Value::Text(reference.source.as_str().to_owned()),
            Value::Bytes(stored_path(&reference.target)?),
            encode_dependencies(dependencies)?,
            candidate_kinds(&reference.target),
            pin.map_or(Value::Null, |id| Value::Text(id.as_str().to_owned())),
            Value::Bytes(authority.to_vec()),
        ]))
    }

    fn authority(&self, reference: &UnresolvedReference) -> Result<[u8; 32], StoreError> {
        self.authority
            .ok_or(StoreError::Invalid(
                "checked facet source authority is absent",
            ))?
            .reference_authority(reference)
    }
}

fn target_key(target: &ReferenceTarget) -> Result<Vec<u8>, StoreError> {
    crate::inventory::work::resource_bytes(&ResourceKey {
        account: target.account().clone(),
        region: target.region().clone(),
        kind: target.kind().clone(),
        id: target.value().clone(),
    })
}

fn candidate_kinds(target: &ReferenceTarget) -> Value {
    if target.candidate_kinds().len() == 1 {
        Value::Null
    } else {
        Value::List(
            target
                .candidate_kinds()
                .iter()
                .map(|kind| Value::Text(kind.as_str().to_owned()))
                .collect(),
        )
    }
}
