use super::{Origin, OverlayError, ServiceOverlay};
use std::collections::BTreeMap;

pub(super) fn merge(
    hand: Vec<ServiceOverlay>,
    generated: Vec<ServiceOverlay>,
) -> Result<Vec<ServiceOverlay>, OverlayError> {
    let mut services = BTreeMap::new();
    for policy in generated {
        if services.insert(policy.service.clone(), policy).is_some() {
            return Err(OverlayError::Invalid(
                "duplicate generated service".to_owned(),
            ));
        }
    }
    for mut policy in hand {
        if let Some(generated) = services.remove(policy.service()) {
            add_generated(&mut policy, generated)?;
        }
        services.insert(policy.service.clone(), policy);
    }
    Ok(services.into_values().collect())
}

fn add_generated(hand: &mut ServiceOverlay, generated: ServiceOverlay) -> Result<(), OverlayError> {
    for (name, operation) in generated.operations {
        if generated_wins(hand.operations.get(&name), &operation) {
            hand.origin.insert(name.clone(), Origin::Generated);
            hand.operations.insert(name, operation);
        }
    }
    if hand.operations.len() > super::MAX_OPERATIONS {
        return Err(OverlayError::Invalid("merged operation limit".to_owned()));
    }
    Ok(())
}

pub(super) fn generated_wins(
    hand: Option<&super::OperationOverlay>,
    generated: &super::OperationOverlay,
) -> bool {
    hand.is_none_or(|entry| {
        entry.class == Some(super::OverlayClass::Defer)
            && entry.generation_blocker == Some(super::GenerationBlocker::RequiredBindings)
            && generated.class == Some(super::OverlayClass::Expand)
    })
}

#[cfg(test)]
mod tests;
