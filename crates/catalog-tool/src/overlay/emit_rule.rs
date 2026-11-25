/// Domain rule for the `emit` overlay key.
///
/// An enumeration that has a single callable expansion bound to it should emit
/// child work instead of resource rows when the list item is a scalar or carries
/// only identifier members. Those items do not themselves have attributes worth
/// storing; only the expansion's output is the inventory record.
///
/// This function is pure and deterministic; its inputs describe the operation's
/// expansion bindings. The item scalar/identifier-only determination is made
/// by the encoder against the actual Smithy model, not by overlay configuration.
use super::SelectionKind;

/// A callable expansion and the available proof of its selected item kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpansionBinding {
    pub output_kind: Option<SelectionKind>,
}

/// Exactly one callable expansion is bound to an enumeration, and it emits a structure.
fn single_structure_expansion(expansions: &[ExpansionBinding]) -> bool {
    expansions.len() == 1
        && expansions
            .first()
            .is_some_and(|expansion| expansion.output_kind == Some(SelectionKind::Structure))
}

/// Pure inference rule for omitting enumeration resource rows.
pub fn infer_work_only(
    item_is_scalar_or_identifier_only: bool,
    expansions: &[ExpansionBinding],
) -> bool {
    item_is_scalar_or_identifier_only && single_structure_expansion(expansions)
}

/// The explicit expansion condition, retained for callers.
pub fn expansion_conditions_met(expansions: &[ExpansionBinding]) -> bool {
    single_structure_expansion(expansions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn structure() -> ExpansionBinding {
        ExpansionBinding {
            output_kind: Some(SelectionKind::Structure),
        }
    }

    #[test]
    fn given_qualifying_item_and_one_structure_expansion_when_inferred_then_work_only() {
        assert!(infer_work_only(true, &[structure()]));
    }

    #[test]
    fn given_unqualified_item_when_inferred_then_resources_remain_enabled() {
        assert!(!infer_work_only(false, &[structure()]));
    }

    #[test]
    fn given_two_structure_expansions_when_inferred_then_resources_remain_enabled() {
        assert!(!infer_work_only(true, &[structure(), structure()]));
    }

    #[test]
    fn given_list_expansion_when_inferred_then_resources_remain_enabled() {
        let expansion = ExpansionBinding {
            output_kind: Some(SelectionKind::List),
        };
        assert!(!infer_work_only(true, &[expansion]));
    }
}
