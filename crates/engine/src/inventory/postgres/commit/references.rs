mod activation;
mod candidates;
mod storage;

use super::keys::Keys;
use super::resources::State;
use crate::inventory::commit::reference_scope;
use crate::inventory::reference_candidates;
use crate::inventory::relation_permissions::{ReferenceProvenance, provenance::proof};
use crate::inventory::{SchemaRegistry, StoreError};
use inventory_core::inventory::{ResourceId, ResourceKey, UnresolvedReference};
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;

pub(super) struct Row<'a> {
    reference: &'a UnresolvedReference,
    target: Option<&'a ResourceId>,
    provenance: &'a ReferenceProvenance,
    scope: Vec<u8>,
    path: Vec<u8>,
    kinds: Vec<u8>,
    proof: [u8; 32],
}

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
    states: &BTreeMap<&ResourceKey, State<'_>>,
) -> Result<(), StoreError> {
    let keys = Keys::new(
        states
            .values()
            .filter(|state| state.facet_changed)
            .map(|state| state.key),
    );
    if keys.ids.is_empty() {
        return Ok(());
    }
    clear(tx, &keys).await?;
    let rows = collect(registry, states)?;
    if !rows.is_empty() {
        storage::persist(tx, &rows).await?;
        activation::persist(tx, &rows).await?;
        candidates::persist(tx, &rows).await?;
    }
    Ok(())
}

fn collect<'a>(
    registry: &SchemaRegistry,
    states: &'a BTreeMap<&ResourceKey, State<'_>>,
) -> Result<Vec<Row<'a>>, StoreError> {
    let mut rows = Vec::new();
    for state in states.values().filter(|state| state.facet_changed) {
        let prepared = state
            .prepared
            .as_ref()
            .ok_or(StoreError::Invalid("resource preparation is absent"))?;
        for (reference, target, provenance) in &prepared.aggregate.references {
            registry.permissions.validate_reference(reference)?;
            rows.push(Row::new(registry, reference, target.as_ref(), provenance)?);
        }
    }
    Ok(rows)
}

impl<'a> Row<'a> {
    fn new(
        registry: &SchemaRegistry,
        reference: &'a UnresolvedReference,
        target: Option<&'a ResourceId>,
        provenance: &'a ReferenceProvenance,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            reference,
            target,
            provenance,
            scope: reference_scope(
                reference.target.account().as_str(),
                reference.target.region().as_str(),
            )?,
            path: reference_candidates::stored_path(&reference.target)?,
            kinds: reference_candidates::encode(&reference.target)?,
            proof: proof(registry, reference),
        })
    }
}

async fn clear(tx: &mut Transaction<'_, Postgres>, keys: &Keys<'_>) -> Result<(), StoreError> {
    for prefix in [
        "DELETE FROM edges e USING ",
        "DELETE FROM unresolved_refs e USING ",
        "DELETE FROM reviewed_reference_evidence e USING ",
    ] {
        let mut query = QueryBuilder::new(prefix);
        keys.bind(&mut query);
        query.push(" u(a,r,k,id) WHERE e.from_account=u.a AND e.from_region=u.r AND e.from_kind=u.k AND e.from_id=u.id");
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}
