use super::SelectionKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpansionBinding {
    pub output_kind: Option<SelectionKind>,
}

fn single_structure_expansion(expansions: &[ExpansionBinding]) -> bool {
    expansions.len() == 1
        && expansions
            .first()
            .is_some_and(|expansion| expansion.output_kind == Some(SelectionKind::Structure))
}

pub fn infer_work_only(
    item_is_scalar_or_identifier_only: bool,
    expansions: &[ExpansionBinding],
) -> bool {
    item_is_scalar_or_identifier_only && single_structure_expansion(expansions)
}

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
