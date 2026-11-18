use super::{CallablePlan, RuntimeCatalogError};
use crate::catalog::runtime::{
    BindingPlan, BindingSource, ContextSource, Emission, IdentityBranchPlan, IdentityChoicePlan,
    IdentityFieldPlan, IdentityLeafPlan, IdentityPredicatePlan, PlanPurpose,
};
use crate::record::FieldId;

pub(super) fn validate(plans: &mut [CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    super::tag_endpoint::validate_parents(plans)?;
    super::routing::validate_plans(plans)?;
    super::context::validate(plans)?;
    super::row_policy::validate_parents(plans)?;
    plans
        .iter()
        .enumerate()
        .filter(|(_, plan)| resource(plan))
        .try_for_each(|(index, plan)| {
            plans
                .iter()
                .take(index)
                .filter(|other| resource(other) && other.kind() == plan.kind())
                .try_for_each(|other| {
                    if same_domain(plan, other) {
                        Ok(())
                    } else {
                        Err(RuntimeCatalogError::Capability)
                    }
                })
        })?;
    let depths = super::stages::depths(plans)?;
    plans.iter_mut().zip(depths).try_for_each(|(plan, depth)| {
        plan.set_facet_precedence(depth);
        Ok(())
    })
}

fn resource(plan: &CallablePlan<'_>) -> bool {
    matches!(plan.purpose(), PlanPurpose::Enumeration) && plan.emission() == Emission::EmitMetadata
}

fn same_domain(left: &CallablePlan<'_>, right: &CallablePlan<'_>) -> bool {
    left.identity() == right.identity()
        || arn_domains(left.identity(), right.identity())
        || echoes_key(left, right)
        || echoes_key(right, left)
        || correlated_domain(left, right)
}

/// Correlated keys share a domain when they differ only in self-context members: each plan
/// reads the same member from the same side of its own operation.
fn correlated_domain(left: &CallablePlan<'_>, right: &CallablePlan<'_>) -> bool {
    let (
        super::IdentityPlan::CorrelatedKey(left_fields),
        super::IdentityPlan::CorrelatedKey(right_fields),
    ) = (left.identity(), right.identity())
    else {
        return false;
    };
    correlated_fields(
        (left.service_name(), left.operation_name()),
        left_fields,
        (right.service_name(), right.operation_name()),
        right_fields,
    )
}

fn correlated_fields(
    left_owner: (&str, &str),
    left: &[IdentityFieldPlan],
    right_owner: (&str, &str),
    right: &[IdentityFieldPlan],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left_field, right_field)| {
            left_field == right_field
                || own_context(left_owner, left_field)
                    .zip(own_context(right_owner, right_field))
                    .is_some_and(|(left_member, right_member)| left_member == right_member)
        })
}

fn own_context<'a>(
    owner: (&str, &str),
    field: &'a IdentityFieldPlan,
) -> Option<(ContextSource, &'a [FieldId])> {
    match field {
        IdentityFieldPlan::Context {
            service,
            operation,
            source,
            path,
        } if (service.as_ref(), operation.as_ref()) == owner => Some((*source, path)),
        _ => None,
    }
}

/// A single-member key is rendered unframed, so a detail keyed by one member shares its
/// parent's key domain exactly when its request sends that parent key through the same member.
fn echoes_key(detail: &CallablePlan<'_>, parent: &CallablePlan<'_>) -> bool {
    let (super::IdentityPlan::Key(detail_key), super::IdentityPlan::Key(parent_key)) =
        (detail.identity(), parent.identity())
    else {
        return false;
    };
    echoes(
        detail_key,
        detail.bindings(),
        (parent.service_name(), parent.operation_name()),
        parent_key,
    )
}

fn echoes(
    detail_key: &[Box<[FieldId]>],
    bindings: &[BindingPlan],
    parent: (&str, &str),
    parent_key: &[Box<[FieldId]>],
) -> bool {
    let ([detail_path], [parent_path]) = (detail_key, parent_key) else {
        return false;
    };
    bindings.iter().any(|binding| {
        binding.path == *detail_path
            && matches!(&binding.source, BindingSource::Operation { service, operation, path }
            if (service.as_ref(), operation.as_ref()) == parent && path == parent_path)
    })
}

/// Two ARN-domain plans agree when their rows that carry no ARN are keyed identically.
fn arn_domains(left: &super::IdentityPlan, right: &super::IdentityPlan) -> bool {
    arn_domain(left)
        && arn_domain(right)
        && (sentinels(left).next().is_none()
            || sentinels(right).next().is_none()
            || sentinels(left).eq(sentinels(right)))
}

fn sentinels(identity: &super::IdentityPlan) -> impl Iterator<Item = &IdentityBranchPlan> {
    let branches: &[IdentityBranchPlan] = match identity {
        super::IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches,
        _ => &[],
    };
    branches.iter().filter(|branch| !leaf_arn(&branch.identity))
}

fn arn_domain(identity: &super::IdentityPlan) -> bool {
    match identity {
        super::IdentityPlan::Arn(_) => true,
        super::IdentityPlan::Template(parts) => {
            matches!(parts.first(), Some(super::IdentityPart::Literal(prefix)) if prefix.starts_with("arn:"))
                && parts
                    .iter()
                    .filter_map(|part| match part {
                        super::IdentityPart::Literal(value) => {
                            Some(value.bytes().filter(|byte| *byte == b':').count())
                        }
                        _ => None,
                    })
                    .sum::<usize>()
                    >= 5
        }
        super::IdentityPlan::MapArn { .. } => true,
        super::IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => {
            leaf_arn(fallback)
        }
        super::IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches
            .iter()
            .all(|branch| leaf_arn(&branch.identity) || arn_sentinel(branch, branches)),
        // A single field is rendered unframed, so one validated ARN operand is the ARN itself.
        super::IdentityPlan::CorrelatedKey(fields) => matches!(
            &**fields,
            [IdentityFieldPlan::SelectedArn(_) | IdentityFieldPlan::RequestArn(_)]
        ),
        super::IdentityPlan::Key(_) => false,
    }
}

/// A branch selected by a literal in the member another branch reads as the ARN covers rows
/// that provably carry no ARN, so no ARN-keyed plan of the kind can name the same resource.
fn arn_sentinel(branch: &IdentityBranchPlan, branches: &[IdentityBranchPlan]) -> bool {
    let IdentityPredicatePlan::TextEquals { path, .. } = &branch.predicate else {
        return false;
    };
    branches
        .iter()
        .any(|other| matches!(&other.identity, IdentityLeafPlan::Arn(arn) if arn == path))
}

fn leaf_arn(identity: &IdentityLeafPlan) -> bool {
    match identity {
        IdentityLeafPlan::Arn(_) => true,
        IdentityLeafPlan::Template(parts) => {
            matches!(parts.first(), Some(super::IdentityPart::Literal(prefix)) if prefix.starts_with("arn:"))
                && parts
                    .iter()
                    .filter_map(|part| match part {
                        super::IdentityPart::Literal(value) => {
                            Some(value.bytes().filter(|byte| *byte == b':').count())
                        }
                        _ => None,
                    })
                    .sum::<usize>()
                    >= 5
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const LIST: (&str, &str) = ("transfer", "ListAccesses");
    const DETAIL: (&str, &str) = ("transfer", "DescribeAccess");

    fn context(
        owner: (&str, &str),
        source: ContextSource,
        member: u32,
    ) -> Result<IdentityFieldPlan, crate::record::RecordError> {
        Ok(IdentityFieldPlan::Context {
            service: owner.0.into(),
            operation: owner.1.into(),
            source,
            path: Box::new([FieldId::new(member)?]),
        })
    }

    fn selected(member: u32) -> Result<IdentityFieldPlan, crate::record::RecordError> {
        Ok(IdentityFieldPlan::Selected(Box::new([FieldId::new(
            member,
        )?])))
    }

    #[test]
    fn given_each_plan_reads_the_same_member_of_its_own_response_when_compared_then_the_domain_is_shared()
    -> TestResult {
        let list = [context(LIST, ContextSource::Response, 7)?, selected(9)?];
        let detail = [context(DETAIL, ContextSource::Response, 7)?, selected(9)?];
        assert!(correlated_fields(LIST, &list, DETAIL, &detail));
        Ok(())
    }

    #[test]
    fn given_self_context_members_or_sides_differ_when_compared_then_the_domain_is_rejected()
    -> TestResult {
        let list = [context(LIST, ContextSource::Response, 7)?, selected(9)?];
        let other_member = [context(DETAIL, ContextSource::Response, 8)?, selected(9)?];
        let other_side = [context(DETAIL, ContextSource::Request, 7)?, selected(9)?];
        let other_selected = [context(DETAIL, ContextSource::Response, 7)?, selected(10)?];
        assert!(!correlated_fields(LIST, &list, DETAIL, &other_member));
        assert!(!correlated_fields(LIST, &list, DETAIL, &other_side));
        assert!(!correlated_fields(LIST, &list, DETAIL, &other_selected));
        assert!(!correlated_fields(
            LIST,
            &list,
            DETAIL,
            other_selected.get(..1).ok_or("prefix absent")?
        ));
        assert!(!correlated_fields(
            LIST,
            list.get(..1).ok_or("prefix absent")?,
            DETAIL,
            &other_selected
        ));
        Ok(())
    }

    #[test]
    fn given_a_context_naming_a_third_operation_when_compared_then_only_exact_equality_is_accepted()
    -> TestResult {
        let foreign = ("transfer", "ListServers");
        let list = [context(foreign, ContextSource::Response, 7)?, selected(9)?];
        let detail = [context(DETAIL, ContextSource::Response, 7)?, selected(9)?];
        assert!(!correlated_fields(LIST, &list, DETAIL, &detail));
        assert!(correlated_fields(LIST, &list, DETAIL, &list));
        assert!(!correlated_fields(DETAIL, &detail, LIST, &list));
        let other_service = [
            context(("ec2", LIST.1), ContextSource::Response, 7)?,
            selected(9)?,
        ];
        assert!(!correlated_fields(LIST, &other_service, DETAIL, &detail));
        Ok(())
    }

    #[test]
    fn given_request_side_or_nested_self_context_when_compared_then_every_path_segment_must_match()
    -> TestResult {
        let nested = |owner: (&str, &str),
                      last: u32|
         -> Result<IdentityFieldPlan, crate::record::RecordError> {
            Ok(IdentityFieldPlan::Context {
                service: owner.0.into(),
                operation: owner.1.into(),
                source: ContextSource::Request,
                path: Box::new([FieldId::new(7)?, FieldId::new(last)?]),
            })
        };
        assert!(correlated_fields(
            LIST,
            &[nested(LIST, 8)?],
            DETAIL,
            &[nested(DETAIL, 8)?]
        ));
        assert!(!correlated_fields(
            LIST,
            &[nested(LIST, 8)?],
            DETAIL,
            &[nested(DETAIL, 9)?]
        ));
        Ok(())
    }

    fn binding(
        member: u32,
        owner: (&str, &str),
        source: u32,
    ) -> Result<BindingPlan, crate::record::RecordError> {
        Ok(BindingPlan {
            path: path(member)?,
            source: BindingSource::Operation {
                service: owner.0.into(),
                operation: owner.1.into(),
                path: path(source)?,
            },
        })
    }

    #[test]
    fn given_a_detail_request_sends_the_parent_key_through_its_key_member_when_compared_then_the_key_is_echoed()
    -> TestResult {
        assert!(echoes(
            &[path(5)?],
            &[binding(5, LIST, 6)?],
            LIST,
            &[path(6)?]
        ));
        Ok(())
    }

    #[test]
    fn given_a_detail_key_not_bound_to_the_parent_key_when_compared_then_the_echo_is_rejected()
    -> TestResult {
        assert!(!echoes(&[path(5)?], &[], LIST, &[path(6)?]));
        assert!(!echoes(
            &[path(5)?, path(6)?],
            &[binding(5, LIST, 6)?],
            LIST,
            &[path(5)?, path(7)?]
        ));
        assert!(!echoes(
            &[path(5)?],
            &[binding(3, LIST, 6)?],
            LIST,
            &[path(6)?]
        ));
        assert!(!echoes(
            &[path(5)?],
            &[binding(5, LIST, 8)?],
            LIST,
            &[path(6)?]
        ));
        assert!(!echoes(
            &[path(5)?],
            &[binding(5, ("transfer", "ListServers"), 6)?],
            LIST,
            &[path(6)?]
        ));
        assert!(!echoes(
            &[path(5)?],
            &[binding(5, ("ec2", LIST.1), 6)?],
            LIST,
            &[path(6)?]
        ));
        Ok(())
    }

    fn path(member: u32) -> Result<Box<[FieldId]>, crate::record::RecordError> {
        Ok(Box::new([FieldId::new(member)?]))
    }

    fn key_branch(
        predicate: IdentityPredicatePlan,
    ) -> Result<IdentityBranchPlan, crate::record::RecordError> {
        Ok(IdentityBranchPlan {
            predicate,
            identity: IdentityLeafPlan::Key(Box::new([path(3)?, path(4)?])),
        })
    }

    fn arn_branch(member: u32) -> Result<IdentityBranchPlan, crate::record::RecordError> {
        Ok(IdentityBranchPlan {
            predicate: IdentityPredicatePlan::Present(path(member)?),
            identity: IdentityLeafPlan::Arn(path(member)?),
        })
    }

    fn discriminated(branches: Vec<IdentityBranchPlan>) -> super::super::IdentityPlan {
        super::super::IdentityPlan::Choice(IdentityChoicePlan::Discriminated(
            branches.into_boxed_slice(),
        ))
    }

    #[test]
    fn given_key_branches_selected_by_a_literal_in_the_arn_member_when_classified_then_the_choice_is_arn_domain()
    -> TestResult {
        let pending = key_branch(IdentityPredicatePlan::TextEquals {
            path: path(5)?,
            value: "PendingConfirmation".into(),
        })?;
        assert!(arn_domain(&discriminated(vec![pending, arn_branch(5)?])));
        Ok(())
    }

    #[test]
    fn given_a_key_branch_not_pinned_to_the_arn_member_when_classified_then_the_choice_is_rejected()
    -> TestResult {
        let other_member = key_branch(IdentityPredicatePlan::TextEquals {
            path: path(6)?,
            value: "PendingConfirmation".into(),
        })?;
        let present = key_branch(IdentityPredicatePlan::Present(path(5)?))?;
        let request = key_branch(IdentityPredicatePlan::RequestTextEquals {
            path: path(5)?,
            value: "PendingConfirmation".into(),
        })?;
        let no_arn_branch = key_branch(IdentityPredicatePlan::TextEquals {
            path: path(5)?,
            value: "PendingConfirmation".into(),
        })?;
        assert!(!arn_domain(&discriminated(vec![
            other_member,
            arn_branch(5)?
        ])));
        assert!(!arn_domain(&discriminated(vec![present, arn_branch(5)?])));
        assert!(!arn_domain(&discriminated(vec![request, arn_branch(5)?])));
        assert!(!arn_domain(&discriminated(vec![no_arn_branch])));
        Ok(())
    }

    #[test]
    fn given_two_sentinel_choices_when_compared_then_their_key_branches_must_be_identical()
    -> TestResult {
        let pending = |members: &[u32]| -> Result<IdentityBranchPlan, crate::record::RecordError> {
            Ok(IdentityBranchPlan {
                predicate: IdentityPredicatePlan::TextEquals {
                    path: path(5)?,
                    value: "PendingConfirmation".into(),
                },
                identity: IdentityLeafPlan::Key(
                    members
                        .iter()
                        .map(|member| path(*member))
                        .collect::<Result<_, _>>()?,
                ),
            })
        };
        let wide = discriminated(vec![pending(&[3, 4])?, arn_branch(5)?]);
        let narrow = discriminated(vec![pending(&[4])?, arn_branch(5)?]);
        let arn_only = super::super::IdentityPlan::Arn(path(5)?);
        assert!(arn_domains(
            &wide,
            &discriminated(vec![pending(&[3, 4])?, arn_branch(5)?])
        ));
        assert!(arn_domains(&wide, &arn_only));
        assert!(arn_domains(&arn_only, &narrow));
        assert!(!arn_domains(&wide, &narrow));
        assert!(!arn_domains(
            &wide,
            &super::super::IdentityPlan::Key(Box::new([path(5)?]))
        ));
        Ok(())
    }

    #[test]
    fn given_a_correlated_key_when_classified_then_only_one_validated_arn_operand_is_arn_domain()
    -> TestResult {
        let correlated = |fields: Vec<IdentityFieldPlan>| {
            super::super::IdentityPlan::CorrelatedKey(fields.into_boxed_slice())
        };
        assert!(arn_domain(&correlated(vec![
            IdentityFieldPlan::RequestArn(path(5)?)
        ])));
        assert!(arn_domain(&correlated(vec![
            IdentityFieldPlan::SelectedArn(path(5)?)
        ])));
        assert!(!arn_domain(&correlated(vec![IdentityFieldPlan::Request(
            path(5)?
        )])));
        assert!(!arn_domain(&correlated(vec![
            IdentityFieldPlan::RequestArn(path(5)?),
            selected(9)?
        ])));
        Ok(())
    }
}
