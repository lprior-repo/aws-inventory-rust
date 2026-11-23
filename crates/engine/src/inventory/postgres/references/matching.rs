mod query;

use super::selection::Reference;
use crate::inventory::{KindSchema, SchemaRegistry, StoreError};
use futures_util::TryStreamExt;
use inventory_core::catalog::runtime::RelationTargetMode;
use inventory_core::inventory::{Kind, ReferenceTarget, ResourceId, ResourceKey};
use inventory_core::record::{FieldId, MapRef, ValueRef};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Resolution {
    pub targets: BTreeSet<ResourceKey>,
    pub ambiguous: bool,
}

struct Request<'a> {
    reference: &'a Reference,
    expected: Option<&'a ResourceId>,
    owner: usize,
    matched: Option<ResourceKey>,
    ambiguous: bool,
}

type Signature<'a> = (&'a Kind, Vec<(u8, &'a [FieldId])>);

pub(super) async fn resolve(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
    references: &[Reference],
) -> Result<Vec<Resolution>, StoreError> {
    let mut requests = requests(references)?;
    let groups = groups(registry, &requests)?;
    for ((kind, _), indexes) in groups {
        let schema = schema(registry, kind)?;
        let Some(mut query) = query::build(schema, &requests, &indexes)? else {
            continue;
        };
        let mut rows = query.build().fetch(&mut **tx);
        while let Some(row) = rows.try_next().await? {
            collect(&mut requests, kind, &row)?;
        }
    }
    resolutions(references.len(), requests)
}

fn schema<'a>(registry: &'a SchemaRegistry, kind: &Kind) -> Result<&'a KindSchema, StoreError> {
    registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == kind)
        .map(|schema| schema.as_ref())
        .ok_or(StoreError::Invalid(
            "referenced kind is absent from schema registry",
        ))
}

fn requests(references: &[Reference]) -> Result<Vec<Request<'_>>, StoreError> {
    let mut requests = Vec::new();
    for (owner, reference) in references.iter().enumerate() {
        for expected in &reference.grants {
            if requests.len() >= 65_536 {
                return Err(StoreError::Invalid(
                    "reference matching input exceeds bounds",
                ));
            }
            requests.push(Request {
                reference,
                expected: expected.as_ref(),
                owner,
                matched: None,
                ambiguous: false,
            });
        }
    }
    Ok(requests)
}

fn groups<'a>(
    registry: &SchemaRegistry,
    requests: &[Request<'a>],
) -> Result<BTreeMap<Signature<'a>, Vec<usize>>, StoreError> {
    let mut groups = BTreeMap::<_, Vec<_>>::new();
    let mut count = 0_usize;
    for (index, request) in requests.iter().enumerate() {
        let reference = request.reference;
        let target = &reference.value.target;
        let parent = registry.permissions.is_parent_reference(&reference.value);
        for kind in target
            .candidate_kinds()
            .iter()
            .filter(|kind| !parent || *kind == target.kind())
        {
            increment(&mut count)?;
            let signature = signature(target);
            groups.entry((kind, signature)).or_default().push(index);
        }
    }
    Ok(groups)
}

fn increment(count: &mut usize) -> Result<(), StoreError> {
    *count = count.checked_add(1).ok_or(StoreError::Invalid(
        "reference matching input exceeds bounds",
    ))?;
    if *count > 65_536 {
        return Err(StoreError::Invalid(
            "reference matching input exceeds bounds",
        ));
    }
    Ok(())
}

fn signature(target: &ReferenceTarget) -> Vec<(u8, &[FieldId])> {
    terms(target)
        .map(|(mode, path, _)| {
            (
                crate::inventory::reference_candidates::mode_code(mode),
                path,
            )
        })
        .collect()
}

fn collect(
    requests: &mut [Request<'_>],
    kind: &Kind,
    row: &sqlx::postgres::PgRow,
) -> Result<(), StoreError> {
    let index = usize::try_from(row.try_get::<i64, _>("reference_index")?)
        .map_err(|_| StoreError::Invalid("reference index exceeds bounds"))?;
    let request = requests
        .get_mut(index)
        .ok_or(StoreError::Invalid("reference index exceeds bounds"))?;
    if request.ambiguous
        || !matches_bound(
            &request.reference.value.target,
            row.try_get("bound_request")?,
        )?
    {
        return Ok(());
    }
    record(request, kind, row.try_get("id")?)
}

fn record(request: &mut Request<'_>, kind: &Kind, id: &str) -> Result<(), StoreError> {
    if let Some(first) = &request.matched {
        request.ambiguous = first.kind != *kind || first.id.as_str() != id;
    } else {
        request.matched = Some(ResourceKey {
            account: request.reference.value.target.account().clone(),
            region: request.reference.value.target.region().clone(),
            kind: kind.clone(),
            id: ResourceId::parse(id)?,
        });
    }
    Ok(())
}

fn resolutions(count: usize, requests: Vec<Request<'_>>) -> Result<Vec<Resolution>, StoreError> {
    let mut resolutions: Vec<_> = std::iter::repeat_with(|| Resolution {
        targets: BTreeSet::new(),
        ambiguous: false,
    })
    .take(count)
    .collect();
    for request in requests {
        let resolution = resolutions
            .get_mut(request.owner)
            .ok_or(StoreError::Invalid("reference index exceeds bounds"))?;
        if request.ambiguous {
            resolution.ambiguous = true;
        } else if let Some(target) = request.matched {
            resolution.targets.insert(target);
        }
    }
    Ok(resolutions)
}

fn terms(
    target: &ReferenceTarget,
) -> impl Iterator<Item = (RelationTargetMode, &[FieldId], &ResourceId)> {
    std::iter::once((target.mode(), target.path(), target.value())).chain(
        target
            .components()
            .iter()
            .map(|term| (term.mode, term.path.as_ref(), &term.value)),
    )
}

fn bound(mode: RelationTargetMode) -> bool {
    matches!(
        mode,
        RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText
    )
}

fn matches_bound(target: &ReferenceTarget, bytes: Option<&[u8]>) -> Result<bool, StoreError> {
    if !terms(target).any(|(mode, _, _)| bound(mode)) {
        return Ok(true);
    }
    let bytes = bytes.ok_or(StoreError::Invalid("bound target context is absent"))?;
    let root = MapRef::from_bytes_bounded(bytes)?;
    terms(target).filter(|(mode, _, _)| bound(*mode)).try_fold(true, |matched, (_, path, value)| {
        Ok(matched && matches!(crate::inventory::relation_permissions::scalar(root, path)?, Some(ValueRef::Text(text)) if text == value.as_str()))
    })
}
