use super::fingerprint::fields;
use super::{ReviewedCompoundRelation, ReviewedParentRelation, ReviewedRelation, ReviewedRelationSource, ReviewedRelationTarget, ReviewedTargetKinds, ReviewedValuePath};
use sha2::Sha256;

pub(super) fn update(
    hash: &mut Sha256,
    relations: &[ReviewedRelation],
    parents: &[ReviewedParentRelation],
    compounds: &[ReviewedCompoundRelation],
) {
    for row in relations {
        if let Some((domain, source, target)) = legacy_fields(row) {
            fields(hash, [domain, &row.source_kind, source, row.target_kinds.primary(), target, &row.relation, &row.scope]);
        } else {
            fields(hash, ["typed_relation", &row.source_kind, &row.relation, &row.scope]);
            source(hash, &row.source);
            target(hash, &row.target);
            targets(hash, &row.target_kinds);
        }
        if let Some(guard) = &row.guard {
            fields(hash, ["source_guard"]);
            source(hash, &guard.source);
            fields(hash, [guard.equals.as_str()]);
        }
    }
    legacy_expanded_groups(hash, relations);
    for row in parents {
        fields(hash, ["parent_relation", &row.source_kind, &row.relation, &row.scope]);
        targets(hash, &row.target_kinds);
    }
    for row in compounds {
        fields(hash, ["compound_relation", &row.source_kind, &row.relation, &row.scope]);
        targets(hash, &row.target_kinds);
        for component in &row.components {
            fields(hash, ["and_component"]);
            source(hash, &component.source);
            target(hash, &component.target);
        }
    }
}

fn legacy_fields(row: &ReviewedRelation) -> Option<(&str, &str, &str)> {
    if row.guard.is_some() {
        return None;
    }
    let ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(target)) = &row.target else {
        return None;
    };
    match &row.source {
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(source)) => Some(("relation", source, target)),
        ReviewedRelationSource::Request(ReviewedValuePath::Fields(source)) => Some(("request_relation", source, target)),
        _ => None,
    }
}

fn legacy_expanded_groups(hash: &mut Sha256, relations: &[ReviewedRelation]) {
    let mut groups = relations.iter().filter(|row| row.target_kinds.len() > 1)
        .filter_map(|row| legacy_fields(row).map(|(_, source, _)| (row, source)));
    match (groups.next(), groups.next()) {
        (Some(first), None) => legacy_group(hash, first),
        (Some(first), Some(second)) => {
            let mut ordered = vec![first, second];
            ordered.extend(groups);
            ordered.sort_by(|left, right| legacy_group_key(*left).cmp(&legacy_group_key(*right)));
            for row in ordered {
                legacy_group(hash, row);
            }
        }
        _ => {}
    }
}

fn legacy_group_key<'a>((row, source): (&'a ReviewedRelation, &'a str)) -> (&'a str, &'a str, &'a str) {
    (&row.source_kind, source, &row.relation)
}

fn legacy_group(hash: &mut Sha256, group: (&ReviewedRelation, &str)) {
    let (kind, path, relation) = legacy_group_key(group);
    fields(hash, ["relation_targets", kind, path, relation]);
    fields(hash, group.0.target_kinds.iter_sorted());
}

fn source(hash: &mut Sha256, source: &ReviewedRelationSource) {
    match source {
        ReviewedRelationSource::SelectedOutput(value) => {
            fields(hash, ["selected_output"]);
            path(hash, value);
        }
        ReviewedRelationSource::Request(value) => {
            fields(hash, ["request"]);
            path(hash, value);
        }
    }
}

fn target(hash: &mut Sha256, target: &ReviewedRelationTarget) {
    match target {
        ReviewedRelationTarget::SelectedOutput(value) => {
            fields(hash, ["selected_output"]);
            path(hash, value);
        }
        ReviewedRelationTarget::BoundContext(value) => {
            fields(hash, ["bound_context"]);
            path(hash, value);
        }
        ReviewedRelationTarget::CanonicalId => fields(hash, ["canonical_id"]),
    }
}

fn path(hash: &mut Sha256, value: &ReviewedValuePath) {
    match value {
        ReviewedValuePath::RootText => fields(hash, ["root_text"]),
        ReviewedValuePath::Fields(value) => fields(hash, ["fields", value]),
    }
}

fn targets(hash: &mut Sha256, values: &ReviewedTargetKinds) {
    fields(hash, ["target_kinds"]);
    fields(hash, values.iter());
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn digest(rows: &[ReviewedRelation], parents: &[ReviewedParentRelation], compounds: &[ReviewedCompoundRelation]) -> [u8; 32] {
        let mut hash = Sha256::new();
        update(&mut hash, rows, parents, compounds);
        hash.finalize().into()
    }

    fn subnet() -> TestResultRow {
        Ok(ReviewedRelation {
            source_kind: "AWS::EC2::Subnet".to_owned(),
            source: ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("VpcId".to_owned())),
            target_kinds: ReviewedTargetKinds::singleton("AWS::EC2::VPC", 0)?,
            target: ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields("VpcId".to_owned())),
            relation: "IN_VPC".to_owned(), scope: "account_region".to_owned(),
            guard: None,
        })
    }

    type TestResultRow = Result<ReviewedRelation, Box<dyn std::error::Error>>;

    #[test]
    fn unchanged_selected_field_relation_retains_the_exact_legacy_hash_preimage() -> TestResult {
        // This byte string is the existing untyped serializer's exact public hash contract.
        let expected: [u8; 32] = Sha256::digest(
            b"relation\0AWS::EC2::Subnet\0VpcId\0AWS::EC2::VPC\0VpcId\0IN_VPC\0account_region\0\xff"
        ).into();
        assert_eq!(digest(&[subnet()?], &[], &[]), expected);
        Ok(())
    }

    #[test]
    fn folded_target_groups_retain_the_exact_legacy_full_member_order() -> TestResult {
        let row = ReviewedRelation {
            source_kind: "AWS::MSK::VpcConnection".to_owned(),
            source: ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("TargetClusterArn".to_owned())),
            target_kinds: ReviewedTargetKinds::from_declared("AWS::MSK::ServerlessCluster",
                &["AWS::MSK::ServerlessCluster", "AWS::MSK::Cluster"], 0)?,
            target: ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields("ClusterArn".to_owned())),
            relation: "CONNECTS_TO".to_owned(), scope: "account_region".to_owned(),
            guard: None,
        };
        // Primary is deliberately not lexically first, catching duplicate-anchor or order drift.
        let mut legacy = Sha256::new();
        legacy.update(b"relation\0AWS::MSK::VpcConnection\0TargetClusterArn\0AWS::MSK::ServerlessCluster\0ClusterArn\0CONNECTS_TO\0account_region\0\xff");
        legacy.update(b"relation_targets\0AWS::MSK::VpcConnection\0TargetClusterArn\0CONNECTS_TO\0\xff");
        legacy.update(b"AWS::MSK::Cluster\0AWS::MSK::ServerlessCluster\0\xff");
        let expected: [u8; 32] = legacy.finalize().into();
        assert_eq!(digest(&[row], &[], &[]), expected);
        Ok(())
    }

    #[test]
    fn multiple_legacy_groups_serialize_in_legacy_label_order_not_relation_traversal_order() -> TestResult {
        let mut last = subnet()?;
        last.relation = "Z_LINK".to_owned();
        last.target_kinds.expand(&["AWS::EC2::VPC", "AWS::ApiOnly::AlternateVPC"], 0)?;
        let mut first = last.clone();
        first.relation = "A_LINK".to_owned();
        let mut actual = Sha256::new();
        legacy_expanded_groups(&mut actual, &[last, first]);
        let actual: [u8; 32] = actual.finalize().into();
        let mut legacy = Sha256::new();
        legacy.update(b"relation_targets\0AWS::EC2::Subnet\0VpcId\0A_LINK\0\xffAWS::ApiOnly::AlternateVPC\0AWS::EC2::VPC\0\xff");
        legacy.update(b"relation_targets\0AWS::EC2::Subnet\0VpcId\0Z_LINK\0\xffAWS::ApiOnly::AlternateVPC\0AWS::EC2::VPC\0\xff");
        let expected: [u8; 32] = legacy.finalize().into();
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn typed_origins_root_text_and_canonical_identity_change_the_review_hash() -> TestResult {
        let row = subnet()?;
        let baseline = digest(std::slice::from_ref(&row), &[], &[]);
        let mut receipts = std::collections::BTreeSet::from([baseline]);
        for changed in [
            ReviewedRelation { source: ReviewedRelationSource::Request(ReviewedValuePath::Fields("VpcId".to_owned())), ..row.clone() },
            ReviewedRelation { source: ReviewedRelationSource::SelectedOutput(ReviewedValuePath::RootText), ..row.clone() },
            ReviewedRelation { target: ReviewedRelationTarget::BoundContext(ReviewedValuePath::Fields("VpcId".to_owned())), ..row.clone() },
            ReviewedRelation { target: ReviewedRelationTarget::CanonicalId, ..row.clone() },
            ReviewedRelation { target: ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::RootText), ..row },
        ] {
            assert!(receipts.insert(digest(&[changed], &[], &[])), "typed selector receipt aliases another namespace");
        }
        Ok(())
    }

    #[test]
    fn unchanged_request_field_relation_retains_the_exact_legacy_hash_preimage() -> TestResult {
        let mut row = subnet()?;
        row.source = ReviewedRelationSource::Request(ReviewedValuePath::Fields("VpcId".to_owned()));
        let expected: [u8; 32] = Sha256::digest(
            b"request_relation\0AWS::EC2::Subnet\0VpcId\0AWS::EC2::VPC\0VpcId\0IN_VPC\0account_region\0\xff"
        ).into();
        assert_eq!(digest(&[row], &[], &[]), expected);
        Ok(())
    }

    #[test]
    fn guard_selector_literal_and_absence_are_bound_without_changing_unguarded_receipts() -> TestResult {
        use super::super::ReviewedRelationGuard;
        let mut row = subnet()?;
        let baseline = digest(std::slice::from_ref(&row), &[], &[]);
        let mut receipts = std::collections::BTreeSet::from([baseline]);
        for source in [
            ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("ResourceType".to_owned())),
            ReviewedRelationSource::Request(ReviewedValuePath::Fields("ResourceType".to_owned())),
            ReviewedRelationSource::SelectedOutput(ReviewedValuePath::RootText),
        ] {
            for equals in ["WORKSPACE", "BUNDLE", "IMAGE", "", "WORKSPACE\0"] {
                row.guard = Some(ReviewedRelationGuard { source: source.clone(), equals: equals.to_owned() });
                assert!(receipts.insert(digest(std::slice::from_ref(&row), &[], &[])));
            }
        }
        row.guard = None;
        assert_eq!(digest(&[row], &[], &[]), baseline);
        Ok(())
    }
}

#[cfg(test)]
#[path = "relation_fingerprint_mode_tests.rs"]
mod mode_tests;
