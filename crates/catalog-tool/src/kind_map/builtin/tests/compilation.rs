use super::*;
#[test]
fn given_embedded_logging_destinations_when_compiled_then_firehose_is_canonical_and_edges_keep_their_selectors()
-> TestResult {
    // Given the real pinned registry and all reviewed native families.
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    // When resolving the destination resource, the exact CFN identity is mapped.
    let kind = "AWS::KinesisFirehose::DeliveryStream";
    assert!(matches!(map.lookup(kind).map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind: target, .. }) if target == kind));
    // Then each logging consumer retains its selected ARN-to-canonical-ID edge.
    for (source, path, scope) in [
        (
            "AWS::VpcLattice::AccessLogSubscription",
            "destinationArn",
            "account_region",
        ),
        (
            "AWS::ApiOnly::WAFLoggingConfiguration",
            "LogDestinationConfigs",
            "arn_region",
        ),
        (
            "AWS::ApiOnly::WAFRegionalLoggingConfiguration",
            "LogDestinationConfigs",
            "account_region",
        ),
    ] {
        let edges = map
            .relations()
            .iter()
            .filter(|row| {
                row.source_kind == source
                    && row.target_kinds.primary() == kind
                    && row.relation == "LOGS_TO"
            })
            .collect::<Vec<_>>();
        assert_eq!(edges.len(), 1, "{source}");
        let edge = edges.first().ok_or("missing Firehose logging edge")?;
        assert!(matches!(&edge.source,
            ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(field)) if field == path));
        assert_eq!(edge.target, ReviewedRelationTarget::CanonicalId);
        assert_eq!(edge.scope, scope, "{source}");
        assert_eq!(edge.guard, None);
    }
    // Alternative destinations must not disappear when the Firehose target is registered.
    let alternatives = map
        .relations()
        .iter()
        .filter(|row| {
            row.source_kind == "AWS::VpcLattice::AccessLogSubscription" && row.relation == "LOGS_TO"
        })
        .map(|row| (row.target_kinds.primary(), row.scope.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        alternatives,
        std::collections::BTreeSet::from([
            ("AWS::KinesisFirehose::DeliveryStream", "account_region"),
            ("AWS::Logs::LogGroup", "account_region"),
            ("AWS::S3::Bucket", "account"),
        ])
    );
    Ok(())
}

#[test]
fn given_s3files_resources_when_embedded_compiles_then_native_operations_and_access_point_edges_share_canonical_kinds()
-> TestResult {
    // Given the real pinned source, when all native mappings compile together.
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    // Then native collection and CloudControl cannot create separate alias nodes.
    for (operation, kind, alias) in [
        (
            "ListFileSystems",
            "AWS::S3Files::FileSystem",
            "AWS::ApiOnly::S3FileSystem",
        ),
        (
            "ListMountTargets",
            "AWS::S3Files::MountTarget",
            "AWS::ApiOnly::S3FileSystemMountTarget",
        ),
    ] {
        assert!(
            matches!(map.lookup(kind).map(|entry| &entry.target),
            Some(KindTarget::Mapped { kind: target, .. }) if target == kind),
            "{kind}"
        );
        assert_eq!(
            map.operations()
                .iter()
                .find(|row| row.service == "s3files" && row.operation == operation)
                .map(|row| row.kind.as_str()),
            Some(kind),
            "{operation}"
        );
        assert!(
            !map.api_only().iter().any(|row| row.kind == alias),
            "{alias}"
        );
        assert!(
            !map.operations().iter().any(|row| row.kind == alias),
            "{alias}"
        );
        assert!(
            !map.relations().iter().any(|row| row.source_kind == alias
                || row.target_kinds.iter().any(|target| target == alias)),
            "{alias}"
        );
    }
    let edges = map
        .relations()
        .iter()
        .filter(|row| {
            row.source_kind == "AWS::S3Files::AccessPoint" && row.relation == "FOR_FILE_SYSTEM"
        })
        .collect::<Vec<_>>();
    assert_eq!(edges.len(), 1);
    let edge = edges
        .first()
        .ok_or("missing access-point filesystem edge")?;
    assert!(matches!(edge.source, ReviewedRelationSource::Request(_)));
    assert_eq!(
        (
            source_fields(edge),
            edge.target_kinds.primary(),
            target_fields(edge),
            edge.scope.as_str()
        ),
        (
            Some("fileSystemId"),
            "AWS::S3Files::FileSystem",
            Some("fileSystemArn"),
            "account_region"
        )
    );
    Ok(())
}

#[test]
fn given_registered_destination_absent_from_pin_when_embedded_compiles_then_exact_extra_type_is_rejected()
-> TestResult {
    // Given a pinned source missing one otherwise registered canonical destination.
    for kind in [
        "AWS::KinesisFirehose::DeliveryStream",
        "AWS::S3Files::FileSystem",
        "AWS::S3Files::MountTarget",
    ] {
        let mut pin: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../../catalog/cloudcontrol-universal.json"
        ))?;
        pin.get_mut("resource_types")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("missing pinned resource types")?
            .retain(|row| row.get("type_name").and_then(serde_json::Value::as_str) != Some(kind));
        // When compiling all families, registration never bypasses pinned-source validation.
        let result = embedded(&serde_json::to_vec(&pin)?);
        // Then the exact missing pinned identity, not an arbitrary replacement, is reported.
        assert!(
            matches!(result, Err(KindMapError::ExtraType(actual)) if actual == kind),
            "{kind}"
        );
    }
    Ok(())
}

#[test]
fn given_unregistered_relation_target_when_compiled_with_pin_then_unknown_kind_is_rejected() {
    use crate::kind_map::{
        CfnMapping, RelationMapping, RelationPath, RelationSource, RelationTarget,
    };
    // Given a valid source kind but no reviewed registration for the target.
    // The first target really exists in the pin; the second is an unknown spelling.
    for target in [
        "AWS::KinesisFirehose::DeliveryStream",
        "AWS::KinesisFirehose::UnknownStream",
    ] {
        let cfn = [CfnMapping::canonical(
            "AWS::VpcLattice::AccessLogSubscription",
        )];
        let relations = [RelationMapping {
            source_kind: "AWS::VpcLattice::AccessLogSubscription",
            source: RelationSource::SelectedOutput(RelationPath::Fields("destinationArn")),
            target_kind: target,
            target: RelationTarget::CanonicalId,
            relation: "LOGS_TO",
            scope: "account_region",
            guard: None,
        }];
        let source = MappingSource {
            cfn: &cfn,
            relations: &relations,
            ..MappingSource::default()
        };
        // When applying the same merged compilation path as embedded().
        let result = super::super::super::merge::compile(
            std::iter::once(source),
            include_bytes!("../../../../../../catalog/cloudcontrol-universal.json"),
        );
        // Then even a pinned target requires an explicit reviewed mapping.
        assert!(
            matches!(result, Err(KindMapError::UnknownRelationKind { relation, kind })
            if relation == "LOGS_TO" && kind == target),
            "{target}"
        );
    }
}

#[test]
fn production_embedded_map_pins_the_complete_api_only_count() -> TestResult {
    // Given the production embedded map compiled from the overlay registry,
    // the api_only count is a complete, reviewed pin.
    let map = crate::kind_map::builtin::embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    let api_only = map.api_only();
    // The production map includes family_d_f rows that the source-only map does not.
    // This count is measured by running this test and observed, not guessed.
    assert_eq!(api_only.len(), 678);
    // DMSCertificate was removed from api_only in this overlay update.
    assert!(
        !api_only
            .iter()
            .any(|r| r.kind == "AWS::ApiOnly::DMSCertificate")
    );
    // The rename from DescribeEventRule to DescribeRule was completed.
    assert!(
        !api_only
            .iter()
            .any(|r| r.reason.contains("DescribeEventRule"))
    );
    Ok(())
}
