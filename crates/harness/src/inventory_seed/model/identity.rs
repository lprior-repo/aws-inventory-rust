use super::*;
pub(in crate::inventory_seed) struct PreparedResource {
    pub kind: Kind,
    pub identity: PreparedIdentity,
    pub scope: super::super::SeedScope,
    pub expected_identity: Option<String>,
}

pub(super) fn resource(
    catalog: Catalog<'_>,
    operation: SeedOperation<'_>,
    step: &SeedStep,
) -> Result<Option<PreparedResource>, SeedError> {
    step.identity
        .as_ref()
        .map(|source| {
            Ok(PreparedResource {
                kind: Kind::parse(&step.kind)?,
                identity: prepare(catalog, operation, source)?,
                scope: step.scope,
                expected_identity: step.expected_identity.clone(),
            })
        })
        .transpose()
}

pub(in crate::inventory_seed) enum PreparedIdentity {
    Response(Vec<Selector>),
    ResponseList {
        list: Vec<Selector>,
        member: Vec<Selector>,
        count: usize,
        token: Option<Continuation>,
    },
    Request {
        input_field: Vec<FieldId>,
        template: Option<String>,
    },
}

pub(in crate::inventory_seed) struct Continuation {
    pub member: String,
    pub path: Vec<Selector>,
}

pub(super) fn prepare(
    catalog: Catalog<'_>,
    operation: SeedOperation<'_>,
    source: &super::super::SeedIdentity,
) -> Result<PreparedIdentity, SeedError> {
    match source {
        super::super::SeedIdentity::Response(path) => Ok(PreparedIdentity::Response(
            output::text_path(operation.service, operation.operation.output()?, path)?,
        )),
        super::super::SeedIdentity::ResponseList {
            list,
            member,
            count,
        } => {
            let target = operation.operation.output()?;
            let (list, element) = output::list_path(operation.service, target, list)?;
            Ok(PreparedIdentity::ResponseList {
                list,
                member: output::text_path(operation.service, element, member)?,
                count: *count,
                token: continuation(catalog, operation, target)?,
            })
        }
        super::super::SeedIdentity::Request {
            input_field,
            template,
        } => request(operation, input_field, template.clone()),
    }
}

fn continuation(
    catalog: Catalog<'_>,
    operation: SeedOperation<'_>,
    target: Target,
) -> Result<Option<Continuation>, SeedError> {
    if let Some(member) = operation
        .operation
        .paginator()?
        .and_then(|paginator| paginator.output_token)
    {
        let path = FieldPath::new(&member.split('.').collect::<Vec<_>>())?;
        return Ok(Some(Continuation {
            member: member.to_owned(),
            path: output::output_path(operation.service, target, &path)?,
        }));
    }
    reject_unmodeled_token(catalog, operation.service, target)?;
    Ok(None)
}

fn reject_unmodeled_token(
    catalog: Catalog<'_>,
    service: Service<'_>,
    target: Target,
) -> Result<(), SeedError> {
    let Target::Shape(index) = target else {
        return Ok(());
    };
    let shape = service
        .shape(index)?
        .ok_or(SeedError::Invalid("seed output shape is missing"))?;
    let members = shape
        .members()?
        .map(|member| member?.name())
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    for service in catalog.services()? {
        for operation in service?.operations()? {
            let token = operation?
                .paginator()?
                .and_then(|paginator| paginator.output_token);
            if let Some(member) = token
                .and_then(|path| path.rsplit('.').next())
                .filter(|name| members.contains(name))
            {
                return Err(SeedError::ResponseListUnmodeledToken {
                    member: member.to_owned(),
                });
            }
        }
    }
    Ok(())
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
