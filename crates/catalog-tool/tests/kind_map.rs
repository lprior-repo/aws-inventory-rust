use catalog_tool::kind_map::{BootstrapPurpose, KindMapError, KindTarget, load, load_with_cfn};
use catalog_tool::names::Exceptions;

const MAP: &str = "meta\tversion\t1\nmeta\treviewed\t2026-11-14\nmeta\treviewer\tAutomated test reviewer\ncfn\tAWS::EC2::VPC\tmap\tAWS::EC2::VPC\t-\ncfn\tAWS::EC2::Subnet\tmap\tAWS::EC2::Subnet\t-\napi\tAWS::ApiOnly::Account\tkind\tBootstrap account identity\napi\tAWS::ApiOnly::Region\tkind\tEnabled region identity\nrelation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\noperation\tec2\tDescribeVpcs\tAWS::EC2::VPC\noperation\tec2\tDescribeSubnets\tAWS::EC2::Subnet\noperation\tec2\tDescribeRegions\tAWS::ApiOnly::Region\noperation\tsts\tGetCallerIdentity\tAWS::ApiOnly::Account\nbootstrap\taccount-identity\tsts\tGetCallerIdentity\tAccount\nbootstrap\tregion-discovery\tec2\tDescribeRegions\tRegions\tRegionName\tOptInStatus\ndefault-unmapped\tNo individually validated mapping exists\n";

fn exceptions() -> Exceptions {
    Exceptions::none()
}

#[test]
fn coverage_expands_explicit_unmapped_reason_and_parses_review_roles() {
    let source = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::S3::Bucket"},{"type_name":"ThirdParty::Widget::Table"}]}"#;
    let result = load_with_cfn(MAP.as_bytes(), source, &exceptions());
    assert!(result.is_ok());
    let Some(map) = result.ok() else {
        return;
    };
    assert_eq!(map.entries().len(), 4);
    assert!(
        matches!(map.lookup("AWS::S3::Bucket").map(|entry| &entry.target), Some(KindTarget::Unmapped { reason }) if reason.contains("No individually"))
    );
    assert!(
        matches!(map.lookup("ThirdParty::Widget::Table").map(|entry| &entry.target), Some(KindTarget::Unmapped { reason }) if reason.contains("No individually"))
    );
    assert_eq!(map.api_only().len(), 2);
    assert_eq!(map.operations().len(), 4);
    assert!(
        matches!(map.bootstrap_for("sts", "GetCallerIdentity"), Some(BootstrapPurpose::AccountIdentity { account_path }) if account_path == "Account")
    );
    assert!(
        matches!(map.bootstrap_for("ec2", "DescribeRegions"), Some(BootstrapPurpose::RegionDiscovery { regions_path, region_name_path, opt_in_status_path }) if regions_path == "Regions" && region_name_path == "RegionName" && opt_in_status_path.as_deref() == Some("OptInStatus"))
    );
    assert!(map.bootstrap_for("ec2", "GetCallerIdentity").is_none());
    assert_eq!(map.source_version(), 1);
    assert!(map.source_sha256().iter().any(|byte| *byte != 0));
    assert!(!map.review_provenance().is_empty());
}

#[test]
fn source_inventory_missing_row_without_fallback_is_rejected() {
    let map = MAP.replace(
        "default-unmapped\tNo individually validated mapping exists\n",
        "",
    );
    let source = br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::Subnet"},{"type_name":"AWS::S3::Bucket"}]}"#;
    assert!(
        matches!(load_with_cfn(map.as_bytes(), source, &exceptions()), Err(KindMapError::MissingType(value)) if value == "AWS::S3::Bucket")
    );
}

#[test]
fn duplicate_source_types_are_rejected() {
    let source =
        br#"{"resource_types":[{"type_name":"AWS::EC2::VPC"},{"type_name":"AWS::EC2::VPC"}]}"#;
    assert!(matches!(
        load_with_cfn(MAP.as_bytes(), source, &exceptions()),
        Err(KindMapError::InvalidRow {
            reason: "duplicate CFN source type",
            ..
        })
    ));
}

#[test]
fn duplicate_cfn_types_are_rejected() {
    let map = MAP.replace(
        "cfn\tAWS::EC2::Subnet\tmap\tAWS::EC2::Subnet\t-\n",
        "cfn\tAWS::EC2::VPC\tmap\tAWS::EC2::VPC\t-\n",
    );
    assert!(matches!(
        load(map.as_bytes(), &exceptions()),
        Err(KindMapError::InvalidRow {
            reason: "invalid or duplicate CFN type",
            ..
        })
    ));
}

#[test]
fn colliding_sql_names_are_rejected() {
    let map = "meta\tversion\t1\nmeta\treviewed\t2026-11-14\nmeta\treviewer\tAutomated test reviewer\ncfn\tAWS::A::B\tmap\tAWS::A::B\t-\ncfn\tAWS::a::b\tmap\tAWS::a::b\t-\n";
    assert!(
        matches!(load(map.as_bytes(), &exceptions()), Err(KindMapError::Duplicate(name)) if name == "aws_a_b")
    );
}

#[test]
fn duplicate_canonical_kind_targets_are_rejected_across_mapped_and_api_only_entries() {
    let duplicate_mapped = MAP.replace(
        "cfn\tAWS::EC2::Subnet\tmap\tAWS::EC2::Subnet\t-",
        "cfn\tAWS::EC2::Subnet\tmap\tAWS::EC2::VPC\t-",
    );
    assert!(
        matches!(load(duplicate_mapped.as_bytes(), &exceptions()), Err(KindMapError::DuplicateCanonicalKind(kind)) if kind == "AWS::EC2::VPC")
    );
    let duplicate_api = MAP.replace(
        "cfn\tAWS::EC2::Subnet\tmap\tAWS::EC2::Subnet\t-",
        "cfn\tAWS::EC2::Subnet\tmap\tAWS::ApiOnly::Region\t-",
    );
    assert!(
        matches!(load(duplicate_api.as_bytes(), &exceptions()), Err(KindMapError::DuplicateCanonicalKind(kind)) if kind == "AWS::ApiOnly::Region")
    );
}

#[test]
fn empty_unmapped_reason_and_bad_model_paths_are_rejected() {
    let no_reason = "meta\tversion\t1\nmeta\treviewed\t2026-11-14\nmeta\treviewer\tAutomated test reviewer\ncfn\tAWS::S3::Bucket\tunmapped\t \t-\n";
    assert!(load(no_reason.as_bytes(), &exceptions()).is_err());
    let bad_path = MAP.replace("VpcId\tAWS::EC2::VPC", "9VpcId\tAWS::EC2::VPC");
    assert!(load(bad_path.as_bytes(), &exceptions()).is_err());
}

#[test]
fn invalid_review_date_and_noncanonical_kind_are_rejected() {
    let bad_date = MAP.replace("2026-11-14", "2026-02-31");
    assert!(load(bad_date.as_bytes(), &exceptions()).is_err());
    let bad_kind = MAP.replace("AWS::EC2::VPC", "AWS::EC2");
    assert!(load(bad_kind.as_bytes(), &exceptions()).is_err());
}

#[test]
fn operation_associations_reject_unknown_kinds_and_duplicate_keys() {
    let unknown = MAP.replace(
        "operation\tec2\tDescribeVpcs\tAWS::EC2::VPC\n",
        "operation\tec2\tDescribeVpcs\tAWS::Missing::Kind\n",
    );
    assert!(
        matches!(load(unknown.as_bytes(), &exceptions()), Err(KindMapError::UnknownKind { kind, .. }) if kind == "AWS::Missing::Kind")
    );
    let duplicate = format!("{MAP}operation\tec2\tDescribeVpcs\tAWS::EC2::VPC\n");
    assert!(
        matches!(load(duplicate.as_bytes(), &exceptions()), Err(KindMapError::Duplicate(key)) if key == "ec2.DescribeVpcs")
    );
}

#[test]
fn bootstrap_roles_are_keyed_to_reviewed_operations_and_api_only_kinds() {
    let duplicate = format!("{MAP}bootstrap\taccount-identity\tsts\tGetCallerIdentity\tAccount\n");
    assert!(
        matches!(load(duplicate.as_bytes(), &exceptions()), Err(KindMapError::Duplicate(key)) if key == "sts.GetCallerIdentity")
    );
    let missing = MAP.replace(
        "bootstrap\taccount-identity\tsts\tGetCallerIdentity\tAccount",
        "bootstrap\taccount-identity\tsts\tMissingOperation\tAccount",
    );
    assert!(
        matches!(load(missing.as_bytes(), &exceptions()), Err(KindMapError::UnknownBootstrapOperation { operation, .. }) if operation == "MissingOperation")
    );
    let not_api = MAP.replace(
        "operation\tec2\tDescribeRegions\tAWS::ApiOnly::Region",
        "operation\tec2\tDescribeRegions\tAWS::EC2::VPC",
    );
    assert!(
        matches!(load(not_api.as_bytes(), &exceptions()), Err(KindMapError::BootstrapOperationNotApiOnly { operation, kind, .. }) if operation == "DescribeRegions" && kind == "AWS::EC2::VPC")
    );
}
