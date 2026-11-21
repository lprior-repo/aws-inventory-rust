mod evidence;
mod matching;
mod typed;
use matching::match_target;
pub(in crate::inventory) use typed::mode_code;
pub(in crate::inventory) use typed::{decode_target, stored_path};

pub(super) use evidence::supports;

use super::{BinaryStore, StoreError, commit};
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::{
    Edge, Kind, ReferenceTarget, ResourceId, ResourceKey, ScalarType, UnresolvedReference,
};
use inventory_core::record::{FieldId, Key, Record, Value, ValueRef};
use sqlx::{Row, Sqlite, Transaction};
use std::collections::BTreeMap;

pub(super) fn encode(target: &ReferenceTarget) -> Result<Vec<u8>, StoreError> {
    if target.candidate_kinds().len() == 1 {
        return Ok(Vec::new());
    }
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        Value::List(
            target
                .candidate_kinds()
                .iter()
                .map(|kind| Value::Text(kind.as_str().to_owned()))
                .collect(),
        ),
    )])))?
    .into_bytes())
}

pub(super) fn decode(bytes: &[u8], primary: &Kind) -> Result<Vec<Kind>, StoreError> {
    if bytes.is_empty() {
        return Ok(vec![primary.clone()]);
    }
    let record = Record::from_bytes(bytes.to_vec())?;
    match record.root().get_field(FieldId::new(1)?)? {
        Some(ValueRef::List(list)) if list.count()? <= 64 => list
            .iter()?
            .map(|value| match value? {
                ValueRef::Text(text) => Ok(Kind::parse(text)?),
                _ => Err(StoreError::Invalid("reference candidate kind is not text")),
            })
            .collect(),
        _ => Err(StoreError::Invalid(
            "reference candidate kinds are absent or excessive",
        )),
    }
}

pub(super) async fn resolve(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<u64, StoreError> {
    let Some(resolution) = current_targets(store, tx, reference).await? else {
        return Ok(0);
    };
    let status = if !resolution.targets.is_empty() {
        "resolved"
    } else if resolution.ambiguous {
        "ambiguous"
    } else {
        "unresolved"
    };
    update_status(tx, reference, status).await?;
    if resolution.ambiguous {
        retract_matches(store, tx, reference).await?;
    }
    let mut inserted = 0_u64;
    for to in resolution.targets {
        if commit::persist_edge(
            tx,
            &Edge {
                from: reference.from.clone(),
                to: to.clone(),
                relation: reference.relation.clone(),
                source: reference.source,
            },
        )
        .await?
        {
            if inserted == 0 {
                commit::enqueue(tx, &reference.from, false).await?;
            }
            commit::enqueue(tx, &to, false).await?;
            inserted = inserted
                .checked_add(1)
                .ok_or(StoreError::Invalid("reference edge count overflow"))?;
        }
    }
    Ok(inserted)
}

struct Resolution {
    targets: Vec<ResourceKey>,
    ambiguous: bool,
}

async fn current_targets(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<Option<Resolution>, StoreError> {
    if !super::relation_permissions::promotable(&store.registry, tx, reference).await? {
        return Ok(None);
    }
    store.registry.permissions.validate_reference(reference)?;
    let grants = super::relation_permissions::targets(&store.registry, tx, reference).await?;
    let mut resolution = Resolution {
        targets: Vec::new(),
        ambiguous: false,
    };
    let only = store
        .registry
        .permissions
        .is_parent_reference(reference)
        .then(|| reference.target.kind());
    for expected in grants {
        match match_target(store, tx, &reference.target, only, expected.as_ref()).await? {
            TargetMatch::Unique(target) if !resolution.targets.contains(&target) => {
                resolution.targets.push(target)
            }
            TargetMatch::Ambiguous => resolution.ambiguous = true,
            TargetMatch::Unique(_) | TargetMatch::Absent => {}
        }
    }
    Ok(Some(resolution))
}

enum TargetMatch {
    Absent,
    Unique(ResourceKey),
    Ambiguous,
}

async fn update_status(
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    status: &str,
) -> Result<(), StoreError> {
    let target = &reference.target;
    // ES-10: Update all rows for this reference (one per candidate kind).
    let path = stored_path(target)?;
    sqlx::query("UPDATE unresolved_refs SET resolution=?1 WHERE from_account=?2 AND from_region=?3 AND from_kind=?4 AND from_id=?5 AND target_scope=?6 AND target_path=?7 AND target_value=?8 AND relation=?9 AND source=?10")
        .bind(status).bind(reference.from.account.as_str()).bind(reference.from.region.as_str())
        .bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(commit::reference_scope(target.account().as_str(), target.region().as_str())?)
        .bind(&path).bind(target.value().as_str()).bind(reference.relation.as_str())
        .bind(reference.source.as_str()).execute(&mut **tx).await?;
    Ok(())
}

async fn retract_matches(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    stream::iter(
        reference
            .target
            .candidate_kinds()
            .iter()
            .map(Ok::<_, StoreError>),
    )
    .try_fold(tx, |tx, kind| async move {
        if !store
            .registry
            .schemas()
            .iter()
            .any(|schema| schema.kind() == kind)
        {
            return Err(StoreError::Invalid(
                "referenced kind is absent from schema registry",
            ));
        }
        retract_kind_matches(store, tx, reference, kind).await?;
        Ok(tx)
    })
    .await?;
    Ok(())
}

async fn retract_kind_matches(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    kind: &Kind,
) -> Result<(), StoreError> {
    // A primary term may live only in checked request context. Enumerate existing
    // edges, not selected columns, and let every independent proof veto retraction.
    stream::try_unfold(Some((tx, None::<ResourceId>)), |state| async move {
        let Some((tx, after)) = state else { return Ok::<_, StoreError>(None); };
        let target = &reference.target;
        let rows = sqlx::query("SELECT to_id FROM edges WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND relation=?5 AND source=?6 AND to_account=?7 AND to_region=?8 AND to_kind=?9 AND to_id>?10 ORDER BY to_id LIMIT 64")
            .bind(reference.from.account.as_str()).bind(reference.from.region.as_str())
            .bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
            .bind(reference.relation.as_str()).bind(reference.source.as_str())
            .bind(target.account().as_str()).bind(target.region().as_str()).bind(kind.as_str())
            .bind(after.as_ref().map_or("", ResourceId::as_str)).fetch_all(&mut **tx).await?;
        let more = rows.len() == 64;
        if rows.is_empty() { return Ok(None); }
        // Finish each bounded query before retraction issues queries on this transaction.
        let (tx, after) = stream::iter(rows.into_iter().map(Ok::<_, StoreError>))
            .try_fold((tx, None), |(tx, _), row| async move {
                let key = ResourceKey { account: target.account().clone(), region: target.region().clone(),
                    kind: kind.clone(), id: ResourceId::parse(row.try_get("to_id")?)? };
                super::references::retract(store, tx, &key, reference).await?;
                Ok((tx, Some(key.id)))
            }).await?;
        Ok(Some(((), if more { Some((tx, after)) } else { None })))
    }).try_for_each(|()| async { Ok(()) }).await
}
