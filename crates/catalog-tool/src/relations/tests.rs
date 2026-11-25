use super::*;
use crate::{
    kind_map::{CfnMapping, KindMap, MappingSource},
    names::Exceptions,
    overlay::{Identity, ItemSelector, MemberPath, Scope},
    runtime_catalog::{ServiceIndex, selection},
};
use inventory_core::catalog::Catalog;
use std::{collections::BTreeMap, path::PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn kinds() -> Result<KindMap, kind_map::KindMapError> {
    MappingSource {
        cfn: &[
            CfnMapping::canonical("AWS::EC2::Subnet"),
            CfnMapping::canonical("AWS::EC2::VPC"),
        ],
        ..MappingSource::default()
    }
    .compile(&Exceptions::none())
}

fn endpoints(identity: &Identity) -> Result<model::Endpoints<'static, '_>, Error> {
    let catalog = Catalog::borrow(include_bytes!("../../../../catalog/aws-api.catalog"))?;
    let index = ServiceIndex::build(catalog)?;
    let service = index.get("ec2")?;
    [
        ("AWS::EC2::Subnet", "DescribeSubnets", "Subnets"),
        ("AWS::EC2::VPC", "DescribeVpcs", "Vpcs"),
    ]
    .into_iter()
    .map(|(kind, operation, items)| {
        let selector =
            ItemSelector::parse(items).map_err(|_| Error::Invalid("fixture selector"))?;
        let item = selection::target(service, model::output(service, operation)?, Some(&selector))?;
        Ok((
            kind,
            vec![model::Endpoint {
                service,
                item,
                identity,
                scope: Some(Scope::Region),
            }],
        ))
    })
    .collect()
}

fn catalogs(pointer: &str) -> Result<Vec<LegacyCatalog>, serde_json::Error> {
    Ok(vec![serde_json::from_value(
        serde_json::json!({"resource_types":[
            {"type_name":"AWS::EC2::Subnet","kind":"aws_ec2_subnet","view_name":"aws_cc_ec2_subnet",
             "relationships":[{"pointer":pointer,"relation":"IN_VPC","target_kind":"aws_ec2_vpc","target_format":"template"}]},
            {"type_name":"AWS::EC2::VPC","kind":"aws_ec2_vpc","view_name":"aws_cc_ec2_vpc"}
        ]}),
    )?])
}

#[test]
fn given_exact_member_when_remapped_then_both_kind_paths_are_exact() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(
        output.rows,
        "relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    assert_eq!(
        output.report,
        "relations mapped_live=1 mapped_held=0 unmapped=0 total=1\ntype\tAWS::EC2::Subnet\tmapped_live=1\tmapped_held=0\tunmapped=0\ntype\tAWS::EC2::VPC\tmapped_live=0\tmapped_held=0\tunmapped=0\nmapped, held: path crosses a list (needs E2.2) count=0\n"
    );
    Ok(())
}

#[test]
fn given_case_only_difference_when_remapped_then_reason_names_the_segment() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let output = remap::build(&catalogs("/vpcId")?, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(output.rows, "");
    assert_eq!(
        output.report,
        "relations mapped_live=0 mapped_held=0 unmapped=1 total=1\ntype\tAWS::EC2::Subnet\tmapped_live=0\tmapped_held=0\tunmapped=1\nunmapped\tAWS::EC2::Subnet\t/vpcId\tIN_VPC\taws_ec2_vpc\ttemplate\tpointer segment vpcId not in item shape\ntype\tAWS::EC2::VPC\tmapped_live=0\tmapped_held=0\tunmapped=0\nmapped, held: path crosses a list (needs E2.2) count=0\nunmapped_reason\tpointer segment vpcId not in item shape\t1\n"
    );
    Ok(())
}

#[test]
fn given_list_pointer_when_resolved_then_wildcard_is_dropped_not_the_member() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let items = endpoints(&identity)?;
    let endpoint = items
        .get("AWS::EC2::Subnet")
        .and_then(|items| items.first())
        .ok_or("missing subnet")?;
    let path = pointer::resolve(
        endpoint.service,
        endpoint.item,
        "/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(path.path, "Ipv6CidrBlockAssociationSet.Ipv6CidrBlock");
    assert_eq!(path.traversal, pointer::Traversal::List);
    let implicit = pointer::resolve(
        endpoint.service,
        endpoint.item,
        "/Ipv6CidrBlockAssociationSet/Ipv6CidrBlock",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(implicit, path);
    let error = pointer::resolve(endpoint.service, endpoint.item, "/VpcId/*")
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "pointer segment * not in item shape");
    let output = remap::build(
        &catalogs("/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock")?,
        &kinds()?,
        &items,
    )?;
    assert_eq!(
        output.rows,
        "relation-held\tAWS::EC2::Subnet\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    assert_eq!(
        output.report,
        "relations mapped_live=0 mapped_held=1 unmapped=0 total=1\ntype\tAWS::EC2::Subnet\tmapped_live=0\tmapped_held=1\tunmapped=0\ntype\tAWS::EC2::VPC\tmapped_live=0\tmapped_held=0\tunmapped=0\nmapped, held: path crosses a list (needs E2.2) count=1\nheld\tAWS::EC2::Subnet\t/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\n"
    );
    let implicit_output = remap::build(
        &catalogs("/Ipv6CidrBlockAssociationSet/Ipv6CidrBlock")?,
        &kinds()?,
        &items,
    )?;
    assert_eq!(implicit_output.rows, output.rows);
    Ok(())
}

#[test]
fn given_inputs_in_reverse_order_when_regenerated_then_both_artifacts_are_identical() -> TestResult
{
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut catalogs = catalogs("/VpcId")?;
    let first = remap::build(&catalogs, &kinds()?, &endpoints(&identity)?)?;
    catalogs
        .iter_mut()
        .for_each(|catalog| catalog.resource_types.reverse());
    let second = remap::build(&catalogs, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(first.rows.as_bytes(), second.rows.as_bytes());
    assert_eq!(first.report.as_bytes(), second.report.as_bytes());
    Ok(())
}

#[test]
fn given_both_reviewed_catalogs_when_reported_then_every_one_of_968_rules_is_counted() -> TestResult
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pinned = std::process::Command::new("git")
        .arg("-C")
        .arg(root.join(".local/aws-models"))
        .args(["rev-parse", "--verify", "HEAD"])
        .output()?;
    assert_eq!(
        pinned.status.code(),
        Some(0),
        "cannot read pinned model commit"
    );
    assert_eq!(
        String::from_utf8(pinned.stdout)?.trim(),
        "46135f8468cd5cb60d983b97290c9f6661e359c6",
        "expected pinned model commit 46135f8468cd5cb60d983b97290c9f6661e359c6"
    );
    let catalogs = read_catalogs(&root)?;
    assert_eq!(
        catalogs
            .iter()
            .map(|catalog| catalog
                .resource_types
                .iter()
                .map(|ty| ty.relationships.len())
                .sum::<usize>())
            .collect::<Vec<_>>(),
        [945, 23]
    );
    let cfn = std::fs::read(root.join("catalog/cloudcontrol-universal.json"))?;
    let kinds = kind_map::load_base(&root, &cfn)?;
    let checked = overlay::check(&root, &root.join(".local/aws-models"))?;
    let bytes = include_bytes!("../../../../catalog/aws-api.catalog");
    let items = model::build(Catalog::borrow(bytes)?, &kinds, &checked)?;
    let output = remap::build(&catalogs, &kinds, &items)?;
    review::assert_real_partition(&output, &kinds, &items)?;
    Ok(())
}

#[test]
fn given_no_callable_source_when_remapped_then_type_is_reported_not_guessed() -> TestResult {
    let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &BTreeMap::new())?;
    assert_eq!(
        output
            .report
            .lines()
            .find(|line| line.starts_with("unmapped\t")),
        Some(
            "unmapped\tAWS::EC2::Subnet\t/VpcId\tIN_VPC\taws_ec2_vpc\ttemplate\ttype not callable"
        )
    );
    Ok(())
}

#[test]
fn given_no_callable_target_when_remapped_then_target_is_reported() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut items = endpoints(&identity)?;
    assert_eq!(
        items.remove("AWS::EC2::VPC").map(|items| items.len()),
        Some(1)
    );
    let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(
        output
            .report
            .lines()
            .find(|line| line.starts_with("unmapped\t")),
        Some(
            "unmapped\tAWS::EC2::Subnet\t/VpcId\tIN_VPC\taws_ec2_vpc\ttemplate\ttarget kind not callable"
        )
    );
    Ok(())
}

#[test]
fn given_arn_identity_when_target_is_arn_then_exact_arn_member_is_used() -> TestResult {
    let identity = Identity::Arn(MemberPath::new("KeyArn")?);
    let catalog = Catalog::borrow(include_bytes!("../../../../catalog/aws-api.catalog"))?;
    let index = ServiceIndex::build(catalog)?;
    let service = index.get("kms")?;
    let selector = ItemSelector::parse("Keys")?;
    let item = selection::target(
        service,
        model::output(service, "ListKeys")?,
        Some(&selector),
    )?;
    let endpoints = BTreeMap::from([(
        "AWS::KMS::Key",
        vec![model::Endpoint {
            service,
            item,
            identity: &identity,
            scope: Some(Scope::Region),
        }],
    )]);
    let kinds = MappingSource {
        cfn: &[CfnMapping::canonical("AWS::KMS::Key")],
        ..MappingSource::default()
    }
    .compile(&Exceptions::none())?;
    let input = vec![serde_json::from_value(
        serde_json::json!({"resource_types":[
            {"type_name":"AWS::KMS::Key","view_name":"aws_kms_key","relationships":[
                {"pointer":"/KeyArn","relation":"REFERENCES","target_kind":"aws_kms_key","target_format":"arn"}
            ]}
        ]}),
    )?];
    let output = remap::build(&input, &kinds, &endpoints)?;
    assert_eq!(
        output.rows,
        "relation\tAWS::KMS::Key\tKeyArn\tAWS::KMS::Key\tKeyArn\tREFERENCES\taccount_region\n"
    );
    Ok(())
}

#[test]
fn given_generic_arn_target_without_matching_identifier_when_remapped_then_unmapped() -> TestResult
{
    let mut input = catalogs("/VpcId")?;
    let rule = input
        .first_mut()
        .and_then(|catalog| catalog.resource_types.first_mut())
        .and_then(|ty| ty.relationships.first_mut())
        .ok_or("missing fixture rule")?;
    rule.target_kind = "aws_reference".to_owned();
    rule.target_format = "arn".to_owned();
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(output.rows, "");
    assert_eq!(
        output
            .report
            .lines()
            .find(|line| line.starts_with("unmapped\t")),
        Some(
            "unmapped\tAWS::EC2::Subnet\t/VpcId\tIN_VPC\taws_reference\tarn\tno enumerated kind has this identifier"
        )
    );
    Ok(())
}

#[test]
fn given_live_held_and_unmapped_rules_when_reported_then_totals_partition_every_rule() -> TestResult
{
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    let ty = input
        .first_mut()
        .and_then(|catalog| catalog.resource_types.first_mut())
        .ok_or("missing source type")?;
    ty.relationships.extend([
        Rule {
            pointer: "/Ipv6CidrBlockAssociationSet/Ipv6CidrBlock".to_owned(),
            relation: "IN_VPC".to_owned(),
            target_kind: "aws_ec2_vpc".to_owned(),
            target_format: "template".to_owned(),
        },
        Rule {
            pointer: "/vpcId".to_owned(),
            relation: "IN_VPC".to_owned(),
            target_kind: "aws_ec2_vpc".to_owned(),
            target_format: "template".to_owned(),
        },
    ]);
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(
        output.report.lines().next(),
        Some("relations mapped_live=1 mapped_held=1 unmapped=1 total=3")
    );
    assert_eq!(
        output
            .report
            .lines()
            .find(|line| line.starts_with("type\tAWS::EC2::Subnet\t")),
        Some("type\tAWS::EC2::Subnet\tmapped_live=1\tmapped_held=1\tunmapped=1")
    );
    assert_eq!(
        output.rows,
        "relation-held\tAWS::EC2::Subnet\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\nrelation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    let loaded = kinds()?.with_generated_relations(output.rows.as_bytes())?;
    assert_eq!(loaded.relations().len(), 1);
    assert_eq!(
        loaded.relations().first().map(|row| &row.source),
        Some(&kind_map::ReviewedRelationSource::SelectedOutput(
            kind_map::ReviewedValuePath::Fields("VpcId".to_owned())
        ))
    );
    Ok(())
}

#[path = "tests/review.rs"]
mod review;

#[path = "tests/pointer_limits.rs"]
mod pointer_limits;
#[path = "tests/round_two.rs"]
mod round_two;
