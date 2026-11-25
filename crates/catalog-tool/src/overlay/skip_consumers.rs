use super::{Binding, CallableOperation, CheckError, CheckedOverlays, policy};

pub(super) fn validate(checked: &CheckedOverlays) -> Result<(), CheckError> {
    checked
        .services()
        .iter()
        .flat_map(|service| service.callable())
        .try_for_each(|parent| {
            parent.plan().skip.iter().try_for_each(|child| {
                let consumer = checked
                    .services()
                    .iter()
                    .flat_map(|service| service.callable())
                    .find(|consumer| consumes(consumer, parent.service(), child.as_str()));
                match consumer {
                    Some(consumer) => Err(CheckError::Invalid {
                        service: parent.service().to_owned(),
                        operation: parent.operation().to_owned(),
                        key: "skip".to_owned(),
                        value: format!(
                            "{child} -> {}/{}",
                            consumer.service(),
                            consumer.operation()
                        ),
                        rule: "skip child has consumers",
                    }),
                    None => Ok(()),
                }
            })
        })
}

fn consumes(consumer: &CallableOperation, service: &str, child: &str) -> bool {
    let plan = consumer.plan();
    let tags = consumer.service() == service
        && plan.tag_endpoint.as_ref().is_some_and(|endpoint| {
            endpoint
                .parents
                .iter()
                .any(|parent| parent.operation.as_str() == child)
        });
    tags || policy::effective_bindings(plan).any(|binding| {
        let source = match binding {
            Binding::Transformed { binding, .. } => binding.as_ref(),
            binding => binding,
        };
        match source {
            Binding::FromService {
                service: producer,
                operation,
                ..
            } => producer == service && operation.as_str() == child,
            Binding::FromAny(alternatives) => {
                consumer.service() == service
                    && alternatives
                        .iter()
                        .any(|source| source.operation.as_str() == child)
            }
            Binding::From { operation, .. }
            | Binding::FromArn { operation, .. }
            | Binding::FromUrl { operation, .. }
            | Binding::Context { operation, .. }
            | Binding::FromBranch { operation, .. }
            | Binding::ContextFromBranch { operation, .. } => {
                consumer.service() == service && operation.as_str() == child
            }
            _ => false,
        }
    })
}
