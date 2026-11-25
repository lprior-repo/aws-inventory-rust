use super::*;
use crate::overlay::{IdentityChoice, IdentityField, IdentityLeaf};

fn source(input: &mut [LegacyCatalog]) -> Result<&mut LegacyType, Box<dyn std::error::Error>> {
    input
        .first_mut()
        .and_then(|catalog| catalog.resource_types.first_mut())
        .ok_or_else(|| "missing source".into())
}

fn first_reason(output: &remap::Output) -> Option<&str> {
    output
        .report
        .lines()
        .find(|line| line.starts_with("unmapped\t"))
        .and_then(|line| line.rsplit('\t').next())
}

#[test]
fn given_incompatible_identity_format_when_remapped_then_target_member_is_unavailable() -> TestResult
{
    for (identity, format) in [
        (Identity::Arn(MemberPath::new("VpcId")?), "template"),
        (
            Identity::Template("arn:{partition}:ec2:{region}:{account}:vpc/{VpcId}".to_owned()),
            "arn",
        ),
        (
            Identity::CorrelatedKey(vec![IdentityField::Selected(MemberPath::new("VpcId")?)]),
            "arn",
        ),
        (
            Identity::Key(vec![
                MemberPath::new("VpcId")?,
                MemberPath::new("CidrBlock")?,
            ]),
            "template",
        ),
        (
            Identity::Choice(IdentityChoice::PreferArn {
                path: MemberPath::new("Arn")?,
                fallback: IdentityLeaf::Arn(MemberPath::new("VpcId")?),
                review: "ARN fallback".to_owned(),
            }),
            "template",
        ),
    ] {
        let mut input = catalogs("/VpcId")?;
        source(&mut input)?
            .relationships
            .first_mut()
            .ok_or("missing rule")?
            .target_format = format.to_owned();
        let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
        assert_eq!(output.rows, "");
        assert_eq!(
            first_reason(&output),
            Some("target ARN or identity member unavailable")
        );
    }
    Ok(())
}

#[test]
fn given_prefer_arn_and_arn_format_when_remapped_then_exact_arn_member_is_used() -> TestResult {
    let identity = Identity::Choice(IdentityChoice::PreferArn {
        path: MemberPath::new("KeyArn")?,
        fallback: IdentityLeaf::Key(vec![MemberPath::new("KeyId")?]),
        review: "selected ARN".to_owned(),
    });
    let catalog = Catalog::borrow(include_bytes!("../../../../../catalog/aws-api.catalog"))?;
    let service = ServiceIndex::build(catalog)?.get("kms")?;
    let selector = ItemSelector::parse("Keys")?;
    let item = selection::target(
        service,
        model::output(service, "ListKeys")?,
        Some(&selector),
    )?;
    let items = BTreeMap::from([(
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
    let output = remap::build(&input, &kinds, &items)?;
    assert_eq!(
        output.rows,
        "relation\tAWS::KMS::Key\tKeyArn\tAWS::KMS::Key\tKeyArn\tREFERENCES\taccount_region\n"
    );
    Ok(())
}

#[test]
fn given_two_pointers_normalising_to_one_held_row_when_remapped_then_only_new_rows_are_counted()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock")?;
    source(&mut input)?.relationships.push(Rule {
        pointer: "/Ipv6CidrBlockAssociationSet/Ipv6CidrBlock".to_owned(),
        relation: "IN_VPC".to_owned(),
        target_kind: "aws_ec2_vpc".to_owned(),
        target_format: "template".to_owned(),
    });
    let first = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(
        first.rows,
        "relation-held\tAWS::EC2::Subnet\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    assert_eq!(
        first.report.lines().next(),
        Some("relations mapped_live=0 mapped_held=1 unmapped=0 total=2")
    );
    assert_eq!(
        first
            .report
            .lines()
            .filter(|line| line.starts_with("held\t"))
            .count(),
        1
    );
    source(&mut input)?.relationships.reverse();
    let second = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(second.rows, first.rows);
    assert_eq!(second.report, first.report);
    Ok(())
}

#[test]
fn given_two_unmapped_and_two_held_rules_when_all_inputs_reverse_then_report_bytes_are_identical()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock")?;
    for pointer in [
        "/Ipv6CidrBlockAssociationSet/*/AssociationId",
        "/unknown",
        "/vpcId",
    ] {
        source(&mut input)?.relationships.push(Rule {
            pointer: pointer.to_owned(),
            relation: "IN_VPC".to_owned(),
            target_kind: "aws_ec2_vpc".to_owned(),
            target_format: "template".to_owned(),
        });
    }
    let first = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(
        first.report.lines().next(),
        Some("relations mapped_live=0 mapped_held=2 unmapped=2 total=4")
    );
    for catalog in &mut input {
        for ty in &mut catalog.resource_types {
            ty.relationships.reverse();
        }
        catalog.resource_types.reverse();
    }
    let second = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(second.rows.as_bytes(), first.rows.as_bytes());
    assert_eq!(second.report.as_bytes(), first.report.as_bytes());
    Ok(())
}

#[test]
fn given_target_endpoint_without_kind_mapping_when_remapped_then_target_is_not_callable()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let kinds = MappingSource {
        cfn: &[CfnMapping::canonical("AWS::EC2::Subnet")],
        ..MappingSource::default()
    }
    .compile(&Exceptions::none())?;
    let output = remap::build(&catalogs("/VpcId")?, &kinds, &endpoints(&identity)?)?;
    assert_eq!(output.rows, "");
    assert_eq!(first_reason(&output), Some("target kind not callable"));
    Ok(())
}
#[test]
fn given_unique_modeled_identifier_when_target_is_generic_then_kind_is_inferred() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    source(&mut input)?
        .relationships
        .first_mut()
        .ok_or("missing relation")?
        .target_kind = "aws_reference".to_owned();
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(
        output.rows,
        "relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    Ok(())
}

#[test]
fn given_identifier_shared_by_enumerated_kinds_when_target_is_generic_then_unmapped() -> TestResult
{
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    source(&mut input)?
        .relationships
        .first_mut()
        .ok_or("missing relation")?
        .target_kind = "aws_reference".to_owned();
    let kinds = MappingSource {
        cfn: &[
            CfnMapping::canonical("AWS::EC2::Subnet"),
            CfnMapping::canonical("AWS::EC2::VPC"),
            CfnMapping::canonical("AWS::EC2::Other"),
        ],
        ..MappingSource::default()
    }
    .compile(&Exceptions::none())?;
    let mut endpoints = endpoints(&identity)?;
    let (service, item, target_identity, scope) = endpoints
        .get("AWS::EC2::VPC")
        .and_then(|items| items.first())
        .map(|endpoint| {
            (
                endpoint.service,
                endpoint.item,
                endpoint.identity,
                endpoint.scope,
            )
        })
        .ok_or("missing VPC endpoint")?;
    endpoints.insert(
        "AWS::EC2::Other",
        vec![model::Endpoint {
            service,
            item,
            identity: target_identity,
            scope,
        }],
    );
    let output = remap::build(&input, &kinds, &endpoints)?;
    assert_eq!(output.rows, "");
    assert_eq!(
        first_reason(&output),
        Some("identifier matches multiple enumerated kinds")
    );
    Ok(())
}
