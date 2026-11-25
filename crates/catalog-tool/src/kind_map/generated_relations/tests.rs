use super::*;
use crate::{
    kind_map::{
        CfnMapping, MappingSource, ReviewedRelationSource, ReviewedRelationTarget,
        ReviewedValuePath,
    },
    names::Exceptions,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const GENERATED: &[u8] =
    b"relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n";

fn given_map(relations: &[RelationMapping<'_>]) -> Result<KindMap, KindMapError> {
    MappingSource {
        cfn: &[
            CfnMapping::canonical("AWS::EC2::Subnet"),
            CfnMapping::canonical("AWS::EC2::VPC"),
        ],
        relations,
        ..MappingSource::default()
    }
    .compile(&Exceptions::none())
}

#[test]
fn given_generated_row_when_loaded_then_both_paths_and_fingerprint_are_retained() -> TestResult {
    let base = given_map(&[])?;
    let before = *base.source_sha256();
    let loaded = base.with_generated_relations(GENERATED)?;
    let row = loaded.relations().first().ok_or("missing generated row")?;
    assert_eq!(loaded.relations().len(), 1);
    assert_eq!(row.source_kind, "AWS::EC2::Subnet");
    assert_eq!(
        row.source,
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields("VpcId".to_owned()))
    );
    assert_eq!(row.target_kinds.primary(), "AWS::EC2::VPC");
    assert_eq!(
        row.target,
        ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields("VpcId".to_owned()))
    );
    assert_eq!(row.relation, "IN_VPC");
    assert_eq!(row.scope, "account_region");
    assert_ne!(*loaded.source_sha256(), before);
    assert_eq!(
        loaded.source_sha256(),
        given_map(&[])?
            .with_generated_relations(GENERATED)?
            .source_sha256()
    );
    Ok(())
}

#[test]
fn given_hand_row_with_same_source_when_loaded_then_hand_target_label_and_scope_win() -> TestResult
{
    let hand = RelationMapping {
        source_kind: "AWS::EC2::Subnet",
        source: RelationSource::SelectedOutput(RelationPath::Fields("VpcId")),
        target_kind: "AWS::EC2::Subnet",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("SubnetId")),
        relation: "REVIEWED",
        scope: "account",
        guard: None,
    };
    let base = given_map(&[hand])?;
    let expected = base.relations().to_vec();
    let fingerprint = *base.source_sha256();
    let loaded = base.with_generated_relations(GENERATED)?;
    assert_eq!(loaded.relations(), expected);
    assert_eq!(*loaded.source_sha256(), fingerprint);
    Ok(())
}

#[test]
fn given_unknown_target_when_loaded_then_error_names_the_missing_kind() -> TestResult {
    let error = given_map(&[])?.with_generated_relations(
        b"relation\tAWS::EC2::Subnet\tVpcId\tAWS::Missing::Target\tId\tIN_VPC\taccount_region\n"
    ).err().ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "relation IN_VPC references unknown kind AWS::Missing::Target"
    );
    Ok(())
}

#[test]
fn given_duplicate_or_malformed_rows_when_loaded_then_exact_error_is_returned() -> TestResult {
    let duplicate = [GENERATED, GENERATED].concat();
    let error = given_map(&[])?
        .with_generated_relations(&duplicate)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid typed kind mapping entry 1: duplicate generated relation"
    );
    let error = given_map(&[])?
        .with_generated_relations(b"relation\tAWS::EC2::Subnet\tVpcId\n")
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid typed kind mapping entry 0: generated relation requires seven tab-separated columns"
    );
    Ok(())
}

#[test]
fn given_more_than_10000_blank_lines_when_loaded_then_line_bound_is_enforced() -> TestResult {
    let bytes = "\n".repeat(10_001);
    let error = given_map(&[])?
        .with_generated_relations(bytes.as_bytes())
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "kind map exceeds the 10000-row bound");
    Ok(())
}

#[test]
fn given_repository_loader_when_generated_file_exists_then_reviewed_source_still_wins() -> TestResult
{
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join("catalog"))?;
    std::fs::write(root.path().join(crate::relations::ROWS_PATH), GENERATED)?;
    let cfn = include_bytes!("../../../../../catalog/cloudcontrol-universal.json");
    let base = crate::kind_map::load_base(root.path(), cfn)?;
    let expected = base.clone().with_generated_relations(GENERATED)?;
    let loaded = crate::kind_map::load(root.path(), cfn)?;
    assert_eq!(loaded.relations(), expected.relations());
    assert_eq!(loaded.source_sha256(), expected.source_sha256());
    assert_eq!(
        loaded
            .relations()
            .iter()
            .filter(|row| row.source_kind == "AWS::EC2::Subnet"
                && row.source
                    == ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(
                        "VpcId".to_owned()
                    )))
            .count(),
        1
    );
    Ok(())
}

#[test]
fn given_held_list_row_when_loaded_then_it_does_not_enter_the_live_kind_map() -> TestResult {
    let base = given_map(&[])?;
    let fingerprint = *base.source_sha256();
    let loaded = base.with_generated_relations(
        b"relation-held\tAWS::EC2::Subnet\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n",
    )?;
    assert_eq!(loaded.relations(), &[]);
    assert_eq!(*loaded.source_sha256(), fingerprint);
    Ok(())
}

#[test]
fn given_exactly_10000_blank_lines_when_loaded_then_line_bound_accepts_input() -> TestResult {
    let base = given_map(&[])?;
    let expected = *base.source_sha256();
    let loaded = base.with_generated_relations("\n".repeat(10_000).as_bytes())?;
    assert_eq!(loaded.relations(), &[]);
    assert_eq!(*loaded.source_sha256(), expected);
    Ok(())
}

#[test]
fn given_missing_generated_file_when_repository_loaded_then_generation_error_is_returned()
-> TestResult {
    let root = tempfile::tempdir()?;
    let error = crate::kind_map::load(
        root.path(),
        include_bytes!("../../../../../catalog/cloudcontrol-universal.json"),
    )
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "catalog/relations.generated.tsv is missing; run catalog-tool relations remap --write"
    );
    Ok(())
}
