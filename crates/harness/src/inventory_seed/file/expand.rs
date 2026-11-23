use super::*;
use std::collections::BTreeMap;

pub(super) struct Bindings<'a> {
    ids: BTreeMap<(&'a str, u16), StepId>,
    descriptors: BTreeMap<&'a str, (usize, u16)>,
}

pub(super) fn convert(document: &Document<'_>) -> Result<SeedScenario, SeedError> {
    super::bounds::validate(document)?;
    let instances = instances(document);
    let bindings = Bindings::new(document, &instances)?;
    let steps = steps(document, &instances, &bindings)?;
    SeedScenario::with_relationships(steps, relationships(document, &bindings)?)
}

pub(super) fn labels(document: &Document<'_>) -> Result<Vec<String>, SeedError> {
    instances(document).iter().map(|(position, index)| {
        let raw = document.steps.get(*position)
            .ok_or(SeedError::Invalid("seed label step is missing"))?;
        let index = index.checked_add(1).ok_or(SeedError::Invalid("seed label overflow"))?;
        Ok(if raw.repeat == 1 { raw.id.to_owned() } else { format!("{}[{index}]", raw.id) })
    }).collect()
}

fn steps(
    document: &Document<'_>,
    instances: &[(usize, u16)],
    bindings: &Bindings<'_>,
) -> Result<Vec<SeedStep>, SeedError> {
    instances
        .iter()
        .enumerate()
        .map(|(id, (position, index))| {
            let raw = document
                .steps
                .get(*position)
                .ok_or(SeedError::Invalid("seed step is missing"))?;
            step::convert(
                document.service,
                raw,
                StepId::try_from(id)?,
                bindings,
                *position,
                *index,
            )
        })
        .collect()
}

fn relationships(
    document: &Document<'_>,
    bindings: &Bindings<'_>,
) -> Result<Vec<SeedRelationship>, SeedError> {
    document
        .relationships
        .iter()
        .try_fold(Vec::new(), |mut output, raw| {
            let count = bindings.repeats(raw.source)?;
            let target = bindings.repeats(raw.target)?;
            if target != 1 && target != count {
                return Err(SeedError::Invalid(
                    "seed relationship repeat counts must match or target one step",
                ));
            }
            (0..count).try_for_each(|index| {
                output.push(relationship(raw, bindings, index)?);
                Ok::<_, SeedError>(())
            })?;
            Ok::<_, SeedError>(output)
        })
}

impl<'a> Bindings<'a> {
    fn new(document: &Document<'a>, instances: &[(usize, u16)]) -> Result<Self, SeedError> {
        let ids = instances
            .iter()
            .enumerate()
            .map(|(id, (position, index))| {
                let raw = document
                    .steps
                    .get(*position)
                    .ok_or(SeedError::Invalid("seed step is missing"))?;
                Ok(((raw.id, *index), StepId::try_from(id)?))
            })
            .collect::<Result<_, SeedError>>()?;
        let descriptors = document
            .steps
            .iter()
            .enumerate()
            .map(|(position, step)| (step.id, (position, step.repeat)))
            .collect();
        Ok(Self { ids, descriptors })
    }

    fn repeats(&self, name: &str) -> Result<u16, SeedError> {
        self.descriptors
            .get(name)
            .map(|(_, repeat)| *repeat)
            .ok_or(SeedError::Invalid("seed reference names an unknown step"))
    }

    fn instance(&self, name: &str, index: u16) -> Result<StepId, SeedError> {
        let index = if self.repeats(name)? == 1 { 0 } else { index };
        self.ids
            .get(&(name, index))
            .copied()
            .ok_or(SeedError::Invalid(
                "seed reference has no matching repeat index",
            ))
    }

    pub fn source(&self, name: &str, position: usize, index: u16) -> Result<StepId, SeedError> {
        let (source, _) = self
            .descriptors
            .get(name)
            .ok_or(SeedError::Invalid("seed reference names an unknown step"))?;
        if *source >= position {
            return Err(SeedError::Invalid(
                "seed binding must refer to an earlier step",
            ));
        }
        self.instance(name, index)
    }
}

fn instances(document: &Document<'_>) -> Vec<(usize, u16)> {
    let maximum = document
        .steps
        .iter()
        .map(|step| step.repeat)
        .fold(0, u16::max);
    (0..maximum)
        .flat_map(|index| {
            document
                .steps
                .iter()
                .enumerate()
                .filter_map(move |(position, step)| {
                    (index < step.repeat).then_some((position, index))
                })
        })
        .collect()
}

fn relationship(
    raw: &RawRelationship<'_>,
    bindings: &Bindings<'_>,
    index: u16,
) -> Result<SeedRelationship, SeedError> {
    Ok(SeedRelationship::new(
        bindings.instance(raw.source, index)?,
        bindings.instance(raw.target, index)?,
        raw.relation.to_owned(),
        path(raw.observed)?,
    ))
}
