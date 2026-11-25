use super::*;
#[test]
fn given_embedded_logging_destinations_when_compiled_then_firehose_is_canonical_and_edges_keep_their_selectors()
-> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    let kind = "AWS::KinesisFirehose::DeliveryStream";
    assert!(matches!(map.lookup(kind).map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind: target, .. }) if target == kind));
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
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
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
        let result = embedded(&serde_json::to_vec(&pin)?);
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
        let result = super::super::super::merge::compile(
            std::iter::once(source),
            include_bytes!("../../../../../../catalog/cloudcontrol-universal.json"),
        );
        assert!(
            matches!(result, Err(KindMapError::UnknownRelationKind { relation, kind })
            if relation == "LOGS_TO" && kind == target),
            "{target}"
        );
    }
}

#[test]
fn production_embedded_map_pins_the_complete_api_only_count() -> TestResult {
    let map = crate::kind_map::builtin::embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    let api_only = map.api_only();
    assert_eq!(api_only.len(), 678);
    assert!(
        !api_only
            .iter()
            .any(|r| r.kind == "AWS::ApiOnly::DMSCertificate")
    );
    assert!(
        !api_only
            .iter()
            .any(|r| r.reason.contains("DescribeEventRule"))
    );
    Ok(())
}
