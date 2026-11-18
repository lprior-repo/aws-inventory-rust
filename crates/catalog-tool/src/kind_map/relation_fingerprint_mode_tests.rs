use super::super::ReviewedRelationComponent;
use super::*;
use sha2::Digest;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CHILD: &str = "AWS::Route53::KeySigningKey";
const ZONE: &str = "AWS::Route53::HostedZone";
const DNSSEC: &str = "AWS::Route53::DNSSEC";
const HEALTH_CHECK: &str = "AWS::Route53::HealthCheck";

fn digest(
    rows: &[ReviewedRelation],
    parents: &[ReviewedParentRelation],
    compounds: &[ReviewedCompoundRelation],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    update(&mut hash, rows, parents, compounds);
    hash.finalize().into()
}

fn parent(members: &[&str]) -> Result<ReviewedParentRelation, super::super::KindMapError> {
    Ok(ReviewedParentRelation {
        source_kind: CHILD.to_owned(),
        target_kinds: ReviewedTargetKinds::from_members(members, 0)?,
        relation: "FOR_HOSTED_ZONE".to_owned(),
        scope: "account".to_owned(),
    })
}

fn compound() -> Result<ReviewedCompoundRelation, super::super::KindMapError> {
    Ok(ReviewedCompoundRelation {
        source_kind: CHILD.to_owned(),
        target_kinds: ReviewedTargetKinds::singleton(ZONE, 0)?,
        relation: "FOR_HOSTED_ZONE".to_owned(),
        scope: "account".to_owned(),
        components: vec![
            ReviewedRelationComponent {
                source: ReviewedRelationSource::Request(ReviewedValuePath::Fields(
                    "AccountId".to_owned(),
                )),
                target: ReviewedRelationTarget::BoundContext(ReviewedValuePath::Fields(
                    "AccountId".to_owned(),
                )),
            },
            ReviewedRelationComponent {
                source: ReviewedRelationSource::Request(ReviewedValuePath::Fields(
                    "HostedZoneId".to_owned(),
                )),
                target: ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(
                    "Id".to_owned(),
                )),
            },
        ],
    })
}

#[test]
fn parent_and_compound_metadata_cannot_share_a_unary_review_receipt() -> TestResult {
    let parent = parent(&[ZONE])?;
    let unary = ReviewedRelation {
        source_kind: CHILD.to_owned(),
        source: ReviewedRelationSource::Request(ReviewedValuePath::Fields(
            "HostedZoneId".to_owned(),
        )),
        target_kinds: parent.target_kinds.clone(),
        target: ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields("Id".to_owned())),
        relation: parent.relation.clone(),
        scope: parent.scope.clone(),
        guard: None,
    };
    let unary_hash = digest(&[unary], &[], &[]);
    let parent_hash = digest(&[], &[parent], &[]);
    let compound_hash = digest(&[], &[], &[compound()?]);
    assert_ne!(unary_hash, parent_hash);
    assert_ne!(unary_hash, compound_hash);
    assert_ne!(parent_hash, compound_hash);
    Ok(())
}

#[test]
fn parent_membership_add_remove_replace_changes_receipts_but_permutation_does_not() -> TestResult {
    let original = parent(&[DNSSEC, ZONE])?;
    let baseline = digest(&[], &[original], &[]);
    let mut receipts = std::collections::BTreeSet::from([baseline]);
    for members in [
        &[DNSSEC][..],
        &[DNSSEC, ZONE, HEALTH_CHECK][..],
        &[DNSSEC, HEALTH_CHECK][..],
    ] {
        let changed = parent(members)?;
        assert_eq!(changed.target_kinds.primary(), DNSSEC);
        assert!(receipts.insert(digest(&[], &[changed], &[])));
    }
    assert_eq!(baseline, digest(&[], &[parent(&[ZONE, DNSSEC])?], &[]));
    Ok(())
}

#[test]
fn compound_source_and_target_namespaces_and_paths_are_all_bound() -> TestResult {
    let original = compound()?;
    let baseline = digest(&[], &[], std::slice::from_ref(&original));
    let mut receipts = std::collections::BTreeSet::from([baseline]);
    for source in [
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("AccountId".to_owned())),
        ReviewedRelationSource::Request(ReviewedValuePath::Fields("AccountNumber".to_owned())),
        ReviewedRelationSource::Request(ReviewedValuePath::RootText),
    ] {
        let mut changed = original.clone();
        changed
            .components
            .first_mut()
            .ok_or("fixture component")?
            .source = source;
        assert!(receipts.insert(digest(&[], &[], &[changed])));
    }
    for target in [
        ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields("AccountId".to_owned())),
        ReviewedRelationTarget::BoundContext(ReviewedValuePath::Fields("AccountNumber".to_owned())),
        ReviewedRelationTarget::CanonicalId,
        ReviewedRelationTarget::BoundContext(ReviewedValuePath::RootText),
    ] {
        let mut changed = original.clone();
        changed
            .components
            .first_mut()
            .ok_or("fixture component")?
            .target = target;
        assert!(receipts.insert(digest(&[], &[], &[changed])));
    }
    Ok(())
}

#[test]
fn adding_and_removing_an_and_component_changes_and_restores_the_receipt() -> TestResult {
    let mut changed = compound()?;
    let baseline = digest(&[], &[], std::slice::from_ref(&changed));
    changed.components.push(ReviewedRelationComponent {
        source: ReviewedRelationSource::Request(ReviewedValuePath::Fields("Region".to_owned())),
        target: ReviewedRelationTarget::BoundContext(ReviewedValuePath::Fields(
            "Region".to_owned(),
        )),
    });
    assert_eq!(changed.components.len(), 3);
    assert_ne!(baseline, digest(&[], &[], std::slice::from_ref(&changed)));
    changed.components.pop().ok_or("fixture component")?;
    assert_eq!(changed.components.len(), 2);
    assert_eq!(baseline, digest(&[], &[], &[changed]));
    Ok(())
}
