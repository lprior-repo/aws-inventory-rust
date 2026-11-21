use catalog_tool::kind_map::{
    ApiOnlyMapping, BootstrapMapping, BootstrapPurpose, CfnMapping, KindMapError, KindTarget,
    MappingSource, OperationMapping, RelationMapping, RelationPath, RelationSource, RelationTarget,
    RelationTargetsMapping, ReviewedRelationSource, ReviewedValuePath,
};
use catalog_tool::names::Exceptions;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const VPC: &str = "AWS::EC2::VPC";
const SUBNET: &str = "AWS::EC2::Subnet";
const ACCOUNT: &str = "AWS::ApiOnly::Account";
const REGION: &str = "AWS::ApiOnly::Region";
const MAP: MappingSource<'static> = MappingSource {
    version: 1,
    reviewed: "2026-11-14",
    reviewer: "Automated test reviewer",
    default_unmapped_reason: Some("No individually validated mapping exists"),
    cfn: &[CfnMapping::canonical(VPC), CfnMapping::canonical(SUBNET)],
    api_only: &[
        ApiOnlyMapping {
            kind: ACCOUNT,
            reason: "Bootstrap account identity",
        },
        ApiOnlyMapping {
            kind: REGION,
            reason: "Enabled region identity",
        },
    ],
    operations: &[
        OperationMapping {
            service: "ec2",
            operation: "DescribeVpcs",
            kind: VPC,
        },
        OperationMapping {
            service: "ec2",
            operation: "DescribeSubnets",
            kind: SUBNET,
        },
        OperationMapping {
            service: "ec2",
            operation: "DescribeRegions",
            kind: REGION,
        },
        OperationMapping {
            service: "sts",
            operation: "GetCallerIdentity",
            kind: ACCOUNT,
        },
    ],
    relations: &[RelationMapping {
        source_kind: SUBNET,
        source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId")),
        target_kind: VPC,
        target: RelationTarget::SelectedOutput(RelationPath::Fields("VpcId")),
        relation: "IN_VPC",
        scope: "account_region",
        guard: None,
    }],
    relation_targets: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    bootstraps: &[
        BootstrapMapping::AccountIdentity {
            service: "sts",
            operation: "GetCallerIdentity",
            account_path: "Account",
        },
        BootstrapMapping::RegionDiscovery {
            service: "ec2",
            operation: "DescribeRegions",
            regions_path: "Regions",
            region_name_path: "RegionName",
            opt_in_status_path: Some("OptInStatus"),
        },
    ],
};

#[test]
fn coverage_expands_explicit_unmapped_reason_and_preserves_review_roles() -> TestResult {
    let source = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::S3::Bucket"},{"type_name":"ThirdParty::Widget::Table"}]}"#;
    let map = MAP.compile_with_cfn(source, &Exceptions::none())?;
    assert_eq!(map.entries().len(), 4);
    ["AWS::S3::Bucket", "ThirdParty::Widget::Table"].iter().for_each(|kind| {
        assert!(matches!(map.lookup(kind).map(|entry| &entry.target), Some(KindTarget::Unmapped { reason }) if reason == "No individually validated mapping exists"));
    });
    assert_eq!(
        (
            map.api_only().len(),
            map.operations().len(),
            map.relations().len()
        ),
        (2, 4, 1)
    );
    assert!(
        matches!(map.bootstrap_for("sts", "GetCallerIdentity"), Some(BootstrapPurpose::AccountIdentity { account_path }) if account_path == "Account")
    );
    assert!(
        matches!(map.bootstrap_for("ec2", "DescribeRegions"), Some(BootstrapPurpose::RegionDiscovery { regions_path, region_name_path, opt_in_status_path }) if regions_path == "Regions" && region_name_path == "RegionName" && opt_in_status_path.as_deref() == Some("OptInStatus"))
    );
    assert!(map.bootstrap_for("ec2", "GetCallerIdentity").is_none());
    assert_eq!(map.source_version(), 1);
    assert_ne!(map.source_sha256(), &[0; 32]);
    assert_eq!(map.review_provenance(), "Automated test reviewer");
    Ok(())
}

#[test]
fn completed_cfn_denominator_changes_fingerprint_but_source_order_does_not() -> TestResult {
    let base =
        br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"}]}"#;
    let expanded = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::S3::Bucket"}]}"#;
    let reordered = br#"{"resource_types":[{"type_name":"AWS::S3::Bucket"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::EC2::VPC"}]}"#;
    let base = MAP.compile_with_cfn(base, &Exceptions::none())?;
    let expanded = MAP.compile_with_cfn(expanded, &Exceptions::none())?;
    let reordered = MAP.compile_with_cfn(reordered, &Exceptions::none())?;
    assert_eq!(base.entries().len(), 2);
    assert_eq!(expanded.entries().len(), 3);
    assert_ne!(base.source_sha256(), expanded.source_sha256());
    assert_eq!(expanded, reordered);
    Ok(())
}

#[test]
fn source_inventory_missing_row_without_fallback_is_rejected() {
    let source = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::S3::Bucket"}]}"#;
    let map = MappingSource {
        default_unmapped_reason: None,
        ..MAP
    };
    assert!(
        matches!(map.compile_with_cfn(source, &Exceptions::none()), Err(KindMapError::MissingType(kind)) if kind == "AWS::S3::Bucket")
    );
}

#[test]
fn duplicate_source_types_are_rejected() {
    let source =
        br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::VPC"}]}"#;
    assert!(matches!(
        MAP.compile_with_cfn(source, &Exceptions::none()),
        Err(KindMapError::InvalidRow {
            reason: "duplicate CFN source type",
            ..
        })
    ));
}

#[test]
fn extra_cfn_types_are_rejected() {
    let source = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"}]}"#;
    assert!(
        matches!(MAP.compile_with_cfn(source, &Exceptions::none()), Err(KindMapError::ExtraType(kind)) if kind == SUBNET)
    );
}

#[test]
fn duplicate_cfn_types_are_rejected() {
    let map = MappingSource {
        cfn: &[CfnMapping::canonical(VPC), CfnMapping::canonical(VPC)],
        ..MAP
    };
    assert!(
        matches!(map.compile(&Exceptions::none()), Err(KindMapError::DuplicateCanonicalKind(kind)) if kind == VPC)
    );
}

#[test]
fn colliding_sql_names_are_rejected() {
    let map = MappingSource {
        cfn: &[
            CfnMapping::canonical("AWS::A::B"),
            CfnMapping::canonical("AWS::a::b"),
        ],
        ..MappingSource::default()
    };
    assert!(
        matches!(map.compile(&Exceptions::none()), Err(KindMapError::Duplicate(name)) if name == "aws_a_b")
    );
}

#[test]
fn duplicate_canonical_targets_across_mapped_and_api_only_are_rejected() {
    [VPC, REGION].iter().for_each(|kind| {
        let entries = [CfnMapping::canonical(VPC), CfnMapping::Mapped { cfn_type: SUBNET, kind, display_name_path: None }];
        let map = MappingSource { cfn: &entries, ..MAP };
        assert!(matches!(map.compile(&Exceptions::none()), Err(KindMapError::DuplicateCanonicalKind(value)) if value == *kind));
    });
}

#[test]
fn empty_reasons_invalid_paths_dates_and_kinds_are_rejected() {
    let no_reason = MappingSource {
        cfn: &[CfnMapping::Unmapped {
            cfn_type: VPC,
            reason: " ",
        }],
        ..MappingSource::default()
    };
    assert!(no_reason.compile(&Exceptions::none()).is_err());
    let no_api_reason = MappingSource {
        api_only: &[ApiOnlyMapping {
            kind: ACCOUNT,
            reason: "",
        }],
        ..MappingSource::default()
    };
    assert!(no_api_reason.compile(&Exceptions::none()).is_err());
    let bad_path = MappingSource {
        cfn: &[CfnMapping::Mapped {
            cfn_type: VPC,
            kind: VPC,
            display_name_path: Some("9Id"),
        }],
        ..MappingSource::default()
    };
    assert!(bad_path.compile(&Exceptions::none()).is_err());
    let bad_date = MappingSource {
        reviewed: "2026-02-31",
        ..MAP
    };
    assert!(bad_date.compile(&Exceptions::none()).is_err());
    let bad_kind = MappingSource {
        cfn: &[CfnMapping::canonical("AWS::EC2")],
        ..MappingSource::default()
    };
    assert!(bad_kind.compile(&Exceptions::none()).is_err());
}

#[test]
fn operation_associations_reject_unknown_kinds_and_duplicate_keys() {
    let unknown = MappingSource {
        operations: &[OperationMapping {
            service: "ec2",
            operation: "DescribeVpcs",
            kind: "AWS::Missing::Kind",
        }],
        bootstraps: &[],
        ..MAP
    };
    assert!(
        matches!(unknown.compile(&Exceptions::none()), Err(KindMapError::UnknownKind { kind, .. }) if kind == "AWS::Missing::Kind")
    );
    let operations = [MAP.operations, MAP.operations.get(..1).unwrap_or_default()].concat();
    let duplicate = MappingSource {
        operations: &operations,
        ..MAP
    };
    assert!(
        matches!(duplicate.compile(&Exceptions::none()), Err(KindMapError::Duplicate(key)) if key == "ec2.DescribeVpcs")
    );
}

#[test]
fn bootstrap_roles_require_unique_reviewed_api_only_operations() {
    let bootstraps = [MAP.bootstraps, MAP.bootstraps.get(..1).unwrap_or_default()].concat();
    let duplicate = MappingSource {
        bootstraps: &bootstraps,
        ..MAP
    };
    assert!(
        matches!(duplicate.compile(&Exceptions::none()), Err(KindMapError::Duplicate(key)) if key == "sts.GetCallerIdentity")
    );
    let missing = MappingSource {
        bootstraps: &[BootstrapMapping::AccountIdentity {
            service: "sts",
            operation: "MissingOperation",
            account_path: "Account",
        }],
        ..MAP
    };
    assert!(
        matches!(missing.compile(&Exceptions::none()), Err(KindMapError::UnknownBootstrapOperation { operation, .. }) if operation == "MissingOperation")
    );
    let operations = MAP
        .operations
        .iter()
        .map(|entry| OperationMapping {
            kind: if entry.operation == "DescribeRegions" {
                VPC
            } else {
                entry.kind
            },
            ..*entry
        })
        .collect::<Vec<_>>();
    let not_api = MappingSource {
        operations: &operations,
        ..MAP
    };
    assert!(
        matches!(not_api.compile(&Exceptions::none()), Err(KindMapError::BootstrapOperationNotApiOnly { operation, kind, .. }) if operation == "DescribeRegions" && kind == VPC)
    );
}

#[test]
fn typed_source_order_does_not_change_mapping_or_fingerprint() -> TestResult {
    let cfn = MAP.cfn.iter().copied().rev().collect::<Vec<_>>();
    let operations = MAP.operations.iter().copied().rev().collect::<Vec<_>>();
    let api = MAP.api_only.iter().copied().rev().collect::<Vec<_>>();
    let reordered = MappingSource {
        cfn: &cfn,
        operations: &operations,
        api_only: &api,
        ..MAP
    };
    assert_eq!(
        MAP.compile(&Exceptions::none())?,
        reordered.compile(&Exceptions::none())?
    );
    Ok(())
}

#[test]
fn reviewed_target_groups_preserve_exact_candidates_and_bind_the_mapping_fingerprint() -> TestResult
{
    // Given one reviewed scalar relation with two explicitly registered target kinds.
    let base = RelationTargetsMapping {
        source_kind: SUBNET,
        source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId")),
        relation: "IN_VPC",
        target_kinds: &[VPC, SUBNET],
    };
    let first = [base];
    let reversed = [RelationTargetsMapping {
        target_kinds: &[SUBNET, VPC],
        ..base
    }];
    // When compiling either input order.
    let baseline = MAP.compile(&Exceptions::none())?;
    let mapped = MappingSource {
        relation_targets: &first,
        ..MAP
    }
    .compile(&Exceptions::none())?;
    let reordered = MappingSource {
        relation_targets: &reversed,
        ..MAP
    }
    .compile(&Exceptions::none())?;
    let relation = mapped
        .relations()
        .first()
        .ok_or("missing fixture relation")?;
    // Then both candidates survive, order is canonical, and the group changes provenance.
    assert_eq!(relation.target_kinds.primary(), VPC);
    assert_eq!(
        relation.target_kinds.iter().collect::<Vec<_>>(),
        [VPC, SUBNET]
    );
    assert_eq!(mapped, reordered);
    assert_ne!(mapped.source_sha256(), baseline.source_sha256());
    Ok(())
}

#[test]
fn reference_target_groups_reject_unknown_duplicate_missing_default_and_orphan_candidates()
-> TestResult {
    let base = RelationTargetsMapping {
        source_kind: SUBNET,
        source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId")),
        relation: "IN_VPC",
        target_kinds: &[VPC, SUBNET],
    };
    let unknown = [RelationTargetsMapping {
        target_kinds: &[VPC, "AWS::Missing::Kind"],
        ..base
    }];
    assert!(
        matches!(MappingSource { relation_targets: &unknown, ..MAP }.compile(&Exceptions::none()),
        Err(KindMapError::UnknownRelationKind { relation, kind }) if relation == "IN_VPC" && kind == "AWS::Missing::Kind")
    );
    let duplicate = [RelationTargetsMapping {
        target_kinds: &[VPC, VPC],
        ..base
    }];
    assert!(
        matches!(MappingSource { relation_targets: &duplicate, ..MAP }.compile(&Exceptions::none()),
        Err(KindMapError::Duplicate(kind)) if kind == VPC)
    );
    let missing_default = [RelationTargetsMapping {
        target_kinds: &[SUBNET, ACCOUNT],
        ..base
    }];
    assert!(matches!(
        MappingSource {
            relation_targets: &missing_default,
            ..MAP
        }
        .compile(&Exceptions::none()),
        Err(KindMapError::InvalidRow {
            line: 0,
            reason: "reviewed target group must include its primary kind within bounds"
        })
    ));
    let orphan = [RelationTargetsMapping {
        source: RelationSource::SelectedOutput(RelationPath::Fields("MissingArn")),
        ..base
    }];
    assert!(matches!(
        MappingSource {
            relation_targets: &orphan,
            ..MAP
        }
        .compile(&Exceptions::none()),
        Err(KindMapError::InvalidRow {
            line: 0,
            reason: "reference target group has no relation"
        })
    ));
    let singleton = [RelationTargetsMapping {
        target_kinds: &[VPC],
        ..base
    }];
    assert!(matches!(
        MappingSource {
            relation_targets: &singleton,
            ..MAP
        }
        .compile(&Exceptions::none()),
        Err(KindMapError::InvalidRow {
            line: 0,
            reason: "invalid reviewed reference target group"
        })
    ));
    let duplicated_group = [base, base];
    assert!(
        matches!(MappingSource { relation_targets: &duplicated_group, ..MAP }.compile(&Exceptions::none()),
        Err(KindMapError::Duplicate(label)) if label == "IN_VPC")
    );
    Ok(())
}

#[test]
fn target_group_bounds_and_reviewed_path_syntax_are_enforced_before_registration() {
    // Given empty, singleton, oversized, and syntactically invalid target group declarations.
    let base = RelationTargetsMapping {
        source_kind: SUBNET,
        source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId")),
        relation: "IN_VPC",
        target_kinds: &[VPC, SUBNET],
    };
    let cases = [
        RelationTargetsMapping {
            target_kinds: &[],
            ..base
        },
        RelationTargetsMapping {
            target_kinds: &[VPC],
            ..base
        },
        RelationTargetsMapping {
            target_kinds: &[VPC; 17],
            ..base
        },
        RelationTargetsMapping {
            source_kind: "not-a-canonical-kind",
            ..base
        },
        RelationTargetsMapping {
            source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId[]")),
            ..base
        },
        RelationTargetsMapping {
            relation: "not a label",
            ..base
        },
    ];
    for group in cases {
        // When compiling the typed source, then it fails with the specific declaration error.
        let rows = [group];
        assert!(
            matches!(
                MappingSource {
                    relation_targets: &rows,
                    ..MAP
                }
                .compile(&Exceptions::none()),
                Err(KindMapError::InvalidRow {
                    line: 0,
                    reason: "invalid reviewed reference target group"
                })
            ),
            "{group:?}"
        );
    }
}

#[test]
fn request_relation_origin_is_explicit_and_changes_the_review_fingerprint() -> TestResult {
    // Given the same parent-link metadata, when its origin is the bound request.
    let selected = MAP.compile(&Exceptions::none())?;
    let relation = MAP
        .relations
        .first()
        .copied()
        .ok_or("missing relation fixture")?;
    let requested = MappingSource {
        relations: &[RelationMapping {
            source: RelationSource::Request(RelationPath::Fields("VpcId")),
            ..relation
        }],
        ..MAP
    }
    .compile(&Exceptions::none())?;
    let selected_row = selected
        .relations()
        .first()
        .ok_or("missing selected relation")?;
    let requested_row = requested
        .relations()
        .first()
        .ok_or("missing request relation")?;
    // Then its typed origin survives and provenance cannot alias selected output.
    assert_eq!(
        selected_row.source,
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("VpcId".to_owned()))
    );
    assert_eq!(
        requested_row.source,
        ReviewedRelationSource::Request(ReviewedValuePath::Fields("VpcId".to_owned()))
    );
    assert_ne!(selected.source_sha256(), requested.source_sha256());
    Ok(())
}

#[test]
fn request_and_selected_sources_remain_distinct_and_target_groups_bind_the_exact_namespace()
-> TestResult {
    // Given two otherwise identical declarations whose only difference is the actual data origin.
    let selected = MAP
        .relations
        .first()
        .copied()
        .ok_or("missing relation fixture")?;
    let requested = RelationMapping {
        source: RelationSource::Request(RelationPath::Fields("VpcId")),
        ..selected
    };
    let relations = [selected, requested];
    let groups = [RelationTargetsMapping {
        source_kind: SUBNET,
        source: requested.source,
        relation: "IN_VPC",
        target_kinds: &[VPC, SUBNET],
    }];
    // When folding an explicitly request-origin group, selected output must stay a singleton.
    let map = MappingSource {
        relations: &relations,
        relation_targets: &groups,
        ..MAP
    }
    .compile(&Exceptions::none())?;
    assert_eq!(map.relations().len(), 2);
    for row in map.relations() {
        match row.source {
            ReviewedRelationSource::SelectedOutput(_) => {
                assert_eq!(row.target_kinds.iter().collect::<Vec<_>>(), [VPC])
            }
            ReviewedRelationSource::Request(_) => {
                assert_eq!(row.target_kinds.iter().collect::<Vec<_>>(), [VPC, SUBNET])
            }
        }
    }
    // Then repeating the exact declaration still fails rather than silently deduplicating.
    let duplicate = [selected, selected];
    assert!(
        matches!(MappingSource { relations: &duplicate, ..MAP }.compile(&Exceptions::none()),
        Err(KindMapError::Duplicate(label)) if label == "IN_VPC")
    );
    Ok(())
}
