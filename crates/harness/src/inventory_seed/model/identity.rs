use super::*;
pub(in crate::inventory_seed) struct PreparedResource {
    pub kind: Kind,
    pub identity: PreparedIdentity,
    pub scope: super::super::SeedScope,
    pub expected_identity: Option<String>,
}

pub(super) fn resource(
    operation: SeedOperation<'_>,
    step: &SeedStep,
) -> Result<Option<PreparedResource>, SeedError> {
    step.identity
        .as_ref()
        .map(|source| {
            Ok(PreparedResource {
                kind: Kind::parse(&step.kind)?,
                identity: prepare(operation, source)?,
                scope: step.scope,
                expected_identity: step.expected_identity.clone(),
            })
        })
        .transpose()
}

pub(in crate::inventory_seed) enum PreparedIdentity {
    Response(Vec<Selector>),
    Request {
        input_field: Vec<FieldId>,
        template: Option<String>,
    },
}

pub(super) fn prepare(
    operation: SeedOperation<'_>,
    source: &super::super::SeedIdentity,
) -> Result<PreparedIdentity, SeedError> {
    match source {
        super::super::SeedIdentity::Response(path) => Ok(PreparedIdentity::Response(
            output::text_path(operation.service, operation.operation.output()?, path)?,
        )),
        super::super::SeedIdentity::Request {
            input_field,
            template,
        } => request(operation, input_field, template.clone()),
    }
}

fn request(
    operation: SeedOperation<'_>,
    field: &FieldPath,
    template: Option<String>,
) -> Result<PreparedIdentity, SeedError> {
    let (input_field, target) = field.parts().iter().try_fold(
        (Vec::new(), operation.operation.input()?),
        |(mut path, target), name| {
            let found = members::member(operation.service, target, name)?;
            path.push(found.id);
            Ok::<_, SeedError>((path, found.target))
        },
    )?;
    output::text_target(operation.service, target)?;
    Ok(PreparedIdentity::Request {
        input_field,
        template,
    })
}
