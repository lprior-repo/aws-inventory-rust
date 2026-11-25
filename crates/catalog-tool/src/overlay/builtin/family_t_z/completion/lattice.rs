use super::*;
use crate::overlay::{IdentityBranch, IdentityChoice, IdentityLeaf, IdentityPredicate};

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    logs_plan(service)?;
    rules_plan(service)?;
    targets_plan(service)
}

fn logs_plan(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut logs = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    logs.class = Some(crate::overlay::OverlayClass::Expand);
    logs.bind.insert(
        path("resourceIdentifier")?,
        Binding::FromAny(
            ["ListServices", "ListServiceNetworks"]
                .into_iter()
                .map(|operation| {
                    Ok(crate::overlay::BindingAlternative {
                        operation: name(operation)?,
                        source: ContextSource::SelectedItem,
                        path: path("arn")?,
                    })
                })
                .collect::<Result<Vec<_>, NameError>>()?,
        ),
    );
    logs.queryable = paths(&["arn", "id", "resourceArn", "resourceId", "destinationArn"])?;
    insert(service, "ListAccessLogSubscriptions", logs)
}

fn rules_plan(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut rules = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    bind(&mut rules, "listenerIdentifier", "ListListeners", "id")?;
    request_context(
        &mut rules,
        "serviceIdentifier",
        "ListListeners",
        "serviceIdentifier",
    )?;
    rules.queryable = paths(&["arn", "id", "name", "priority", "isDefault"])?;
    insert(service, "ListRules", rules)
}

fn targets_plan(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let with_port = vec![
        IdentityField::Request(path("targetGroupIdentifier")?),
        IdentityField::Selected(path("id")?),
        IdentityField::Selected(path("port")?),
    ];
    let without_port = vec![
        IdentityField::Request(path("targetGroupIdentifier")?),
        IdentityField::Selected(path("id")?),
    ];
    let identity = Identity::Choice(IdentityChoice::Discriminated(vec![
        IdentityBranch::new(
            IdentityPredicate::Present(path("port")?),
            IdentityLeaf::Fields(with_port),
        )?,
        IdentityBranch::new(
            IdentityPredicate::Present(path("id")?),
            IdentityLeaf::Fields(without_port),
        )?,
    ]));
    let mut targets = inventory(
        "items",
        identity,
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    bind(
        &mut targets,
        "targetGroupIdentifier",
        "ListTargetGroups",
        "arn",
    )?;
    targets.queryable = paths(&["id", "port", "status", "reasonCode"])?;
    insert(service, "ListTargets", targets)
}
