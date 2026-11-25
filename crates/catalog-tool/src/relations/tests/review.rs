use super::*;
use crate::overlay::{IdentityChoice, IdentityField, IdentityLeaf};
use inventory_core::catalog::{CatalogError, Target};

const ROW: &str =
    "relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n";

fn rule(input: &mut [LegacyCatalog]) -> Result<&mut Rule, Box<dyn std::error::Error>> {
    input
        .first_mut()
        .and_then(|catalog| catalog.resource_types.first_mut())
        .and_then(|ty| ty.relationships.first_mut())
        .ok_or_else(|| "missing rule".into())
}

fn reason(output: &remap::Output) -> Option<&str> {
    output
        .report
        .lines()
        .find(|line| line.starts_with("unmapped\t"))
        .and_then(|line| line.rsplit('\t').next())
}

pub(super) fn assert_real_partition(
    output: &remap::Output,
    kinds: &KindMap,
    items: &model::Endpoints<'_, '_>,
) -> TestResult {
    let header = output.report.lines().next().ok_or("missing totals")?;
    assert_eq!(header.split(' ').next_back(), Some("total=968"));
    let counts = parse_counts(header)?;
    assert_eq!(counts.iter().copied().sum::<usize>(), 968);
    assert_ne!(counts.first(), Some(&0));
    assert_ne!(counts.get(1), Some(&0));
    let totals = output
        .report
        .lines()
        .filter(|line| line.starts_with("type\t"))
        .try_fold([0usize; 3], |mut total, line| {
            for (sum, count) in total.iter_mut().zip(parse_counts(line)?) {
                *sum = sum.checked_add(count).ok_or("count overflow")?;
            }
            Ok::<_, Box<dyn std::error::Error>>(total)
        })?;
    assert_eq!(totals, counts);
    assert_eq!(
        output
            .report
            .lines()
            .filter(|line| line.starts_with("unmapped\t"))
            .count(),
        counts.get(2).copied().ok_or("missing unmapped count")?
    );
    let live = output
        .rows
        .lines()
        .filter(|line| line.starts_with("relation\t"))
        .count();
    let held = output
        .rows
        .lines()
        .filter(|line| line.starts_with("relation-held\t"))
        .count();
    assert_eq!(Some(&live), counts.first());
    assert_eq!(Some(&held), counts.get(1));
    for row in output.rows.lines() {
        let target = row.split('\t').nth(3).ok_or("missing target")?;
        assert!(items.contains_key(target), "{target}");
        assert!(
            kinds.entries().iter().any(|entry| matches!(&entry.target,
            kind_map::KindTarget::Mapped { kind, .. } if kind == target)),
            "{target}"
        );
    }
    kinds
        .clone()
        .with_generated_relations(output.rows.as_bytes())?;
    Ok(())
}

fn parse_counts(line: &str) -> Result<[usize; 3], Box<dyn std::error::Error>> {
    let [live, held, unmapped]: [Result<usize, Box<dyn std::error::Error>>; 3] =
        ["mapped_live=", "mapped_held=", "unmapped="].map(|prefix| {
            line.split(['\t', ' '])
                .find_map(|cell| cell.strip_prefix(prefix))
                .ok_or("missing count")?
                .parse::<usize>()
                .map_err(Into::into)
        });
    Ok([live?, held?, unmapped?])
}

#[test]
fn given_account_and_home_region_targets_when_remapped_then_exact_scope_is_emitted() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    for (scope, expected) in [
        (Scope::Account, "account"),
        (Scope::HomeRegion, "home_region"),
    ] {
        let mut items = endpoints(&identity)?;
        items
            .get_mut("AWS::EC2::VPC")
            .and_then(|items| items.first_mut())
            .ok_or("missing target")?
            .scope = Some(scope);
        let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
        assert_eq!(
            output.rows,
            format!(
                "relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\t{expected}\n"
            )
        );
    }
    Ok(())
}

#[test]
fn given_template_and_correlated_identities_when_remapped_then_selected_member_is_used()
-> TestResult {
    for identity in [
        Identity::Template("arn:{partition}:ec2:{region}:{account}:vpc/{VpcId}".to_owned()),
        Identity::CorrelatedKey(vec![IdentityField::Selected(MemberPath::new("VpcId")?)]),
    ] {
        let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &endpoints(&identity)?)?;
        assert_eq!(output.rows, ROW);
    }
    Ok(())
}

#[test]
fn given_prefer_arn_when_target_format_is_not_arn_then_each_fallback_leaf_is_used() -> TestResult {
    for fallback in [
        IdentityLeaf::Key(vec![MemberPath::new("VpcId")?]),
        IdentityLeaf::Template("arn:{partition}:ec2:{region}:{account}:vpc/{VpcId}".to_owned()),
        IdentityLeaf::Fields(vec![IdentityField::Selected(MemberPath::new("VpcId")?)]),
    ] {
        let identity = Identity::Choice(IdentityChoice::PreferArn {
            path: MemberPath::new("Arn")?,
            fallback,
            review: "selected fallback".to_owned(),
        });
        let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &endpoints(&identity)?)?;
        assert_eq!(output.rows, ROW);
    }
    Ok(())
}

#[test]
fn given_arn_format_with_key_identity_when_remapped_then_unavailable_member_is_reported()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    rule(&mut input)?.target_format = "arn".to_owned();
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(output.rows, "");
    assert_eq!(
        reason(&output),
        Some("target ARN or identity member unavailable")
    );
    Ok(())
}

#[test]
fn given_view_name_alias_when_remapped_then_target_kind_resolves() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    rule(&mut input)?.target_kind = "aws_cc_ec2_vpc".to_owned();
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(output.rows, ROW);
    Ok(())
}

#[test]
fn given_unknown_legacy_names_when_remapped_then_unknown_is_distinct_from_not_callable()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    for name in ["aws_iam_role", "aws_ec2_image"] {
        let mut input = catalogs("/VpcId")?;
        rule(&mut input)?.target_kind = name.to_owned();
        let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
        assert_eq!(output.rows, "");
        assert_eq!(reason(&output), Some("unresolved legacy target name"));
    }
    Ok(())
}

#[test]
fn given_colliding_legacy_name_when_remapped_then_only_affected_rule_is_unmapped() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    let types = &mut input.first_mut().ok_or("missing catalog")?.resource_types;
    types.first_mut().ok_or("missing source")?.view_name = "aws_cc_ec2_vpc".to_owned();
    let source = types.first_mut().ok_or("missing source")?;
    source.relationships.push(Rule {
        pointer: "/SubnetId".to_owned(),
        relation: "ALIAS".to_owned(),
        target_kind: "aws_cc_ec2_vpc".to_owned(),
        target_format: "template".to_owned(),
    });
    let output = remap::build(&input, &kinds()?, &endpoints(&identity)?)?;
    assert_eq!(output.rows, ROW);
    assert_eq!(reason(&output), Some("ambiguous legacy target name"));
    assert_eq!(
        output.report.lines().next(),
        Some("relations mapped_live=1 mapped_held=0 unmapped=1 total=2")
    );
    Ok(())
}

#[test]
fn given_duplicate_source_rule_when_remapped_then_duplicate_is_rejected() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut input = catalogs("/VpcId")?;
    let ty = input
        .first_mut()
        .and_then(|catalog| catalog.resource_types.first_mut())
        .ok_or("missing source")?;
    ty.relationships.push(Rule {
        pointer: "/VpcId".to_owned(),
        relation: "IN_VPC".to_owned(),
        target_kind: "aws_ec2_vpc".to_owned(),
        target_format: "template".to_owned(),
    });
    let error = remap::build(&input, &kinds()?, &endpoints(&identity)?)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid relation input: duplicate source rule"
    );
    Ok(())
}

#[test]
fn given_invalid_or_non_text_pointer_when_resolved_then_exact_reason_is_returned() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let items = endpoints(&identity)?;
    let endpoint = items
        .get("AWS::EC2::Subnet")
        .and_then(|items| items.first())
        .ok_or("missing source")?;
    let oversized = format!("/{}", "x".repeat(1024));
    for (pointer, expected) in [
        (
            "/Ipv6CidrBlockAssociationSet",
            "pointer does not end in text",
        ),
        ("/Vpc~1Id", "pointer segment Vpc/Id not in item shape"),
        ("/Vpc~0Id", "pointer segment Vpc~Id not in item shape"),
        ("/Vpc~2Id", "invalid pointer"),
        ("", "invalid pointer"),
        ("/", "invalid pointer"),
        (oversized.as_str(), "invalid pointer"),
    ] {
        let error = pointer::resolve(endpoint.service, endpoint.item, pointer)
            .err()
            .ok_or("expected an error")?;
        assert_eq!(error.to_string(), expected);
    }
    Ok(())
}

fn nested_item(
    service: inventory_core::catalog::Service<'_>,
    target: Target,
    name: &str,
) -> Result<Target, Box<dyn std::error::Error>> {
    let Target::Shape(index) = target else {
        return Err("expected structure".into());
    };
    service
        .shape(index)?
        .ok_or("missing structure")?
        .members()?
        .try_fold(None, |found, member| {
            let member = member?;
            Ok::<_, CatalogError>(if member.name()? == name {
                Some(member.target)
            } else {
                found
            })
        })?
        .ok_or_else(|| "missing member".into())
}

#[test]
fn given_source_endpoints_disagree_on_list_traversal_when_reordered_then_rule_stays_ambiguous()
-> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut items = endpoints(&identity)?;
    let sources = items.get_mut("AWS::EC2::Subnet").ok_or("missing source")?;
    let source = sources.first_mut().ok_or("missing endpoint")?;
    let list = nested_item(source.service, source.item, "Ipv6CidrBlockAssociationSet")?;
    let Target::Shape(index) = list else {
        return Err("expected list".into());
    };
    let element = source
        .service
        .shape(index)?
        .ok_or("missing list")?
        .element()?
        .ok_or("missing element")?;
    source.item = list;
    let scalar = model::Endpoint {
        service: source.service,
        item: element,
        identity: &identity,
        scope: source.scope,
    };
    sources.push(scalar);
    let input = catalogs("/Ipv6CidrBlock")?;
    let first = remap::build(&input, &kinds()?, &items)?;
    assert_eq!(first.rows, "");
    assert_eq!(reason(&first), Some("ambiguous source endpoints"));
    items
        .get_mut("AWS::EC2::Subnet")
        .ok_or("missing source")?
        .reverse();
    let second = remap::build(&input, &kinds()?, &items)?;
    assert_eq!(second.report, first.report);
    Ok(())
}

#[test]
fn given_target_endpoints_disagree_on_scope_when_reordered_then_rule_stays_ambiguous() -> TestResult
{
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut items = endpoints(&identity)?;
    let targets = items.get_mut("AWS::EC2::VPC").ok_or("missing target")?;
    let target = targets.first().ok_or("missing endpoint")?;
    let other = model::Endpoint {
        service: target.service,
        item: target.item,
        identity: &identity,
        scope: Some(Scope::Account),
    };
    targets.push(other);
    let first = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(first.rows, "");
    assert_eq!(reason(&first), Some("ambiguous target endpoints"));
    items
        .get_mut("AWS::EC2::VPC")
        .ok_or("missing target")?
        .reverse();
    let second = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(second.report, first.report);
    Ok(())
}

#[test]
fn given_identical_endpoints_when_remapped_then_one_live_row_is_emitted() -> TestResult {
    let identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let mut items = endpoints(&identity)?;
    for kind in ["AWS::EC2::Subnet", "AWS::EC2::VPC"] {
        let endpoints = items.get_mut(kind).ok_or("missing kind")?;
        let first = endpoints.first().ok_or("missing endpoint")?;
        let duplicate = model::Endpoint {
            service: first.service,
            item: first.item,
            identity: &identity,
            scope: first.scope,
        };
        endpoints.push(duplicate);
    }
    let output = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(output.rows, ROW);
    assert_eq!(
        output.report.lines().next(),
        Some("relations mapped_live=1 mapped_held=0 unmapped=0 total=1")
    );
    Ok(())
}

#[test]
fn given_target_endpoints_disagree_on_identity_member_when_reordered_then_rule_stays_ambiguous()
-> TestResult {
    let first_identity = Identity::Key(vec![MemberPath::new("VpcId")?]);
    let other_identity = Identity::Key(vec![MemberPath::new("CidrBlock")?]);
    let mut items = endpoints(&first_identity)?;
    let targets = items.get_mut("AWS::EC2::VPC").ok_or("missing target")?;
    let first = targets.first().ok_or("missing endpoint")?;
    let other = model::Endpoint {
        service: first.service,
        item: first.item,
        identity: &other_identity,
        scope: first.scope,
    };
    targets.push(other);
    let first = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(first.rows, "");
    assert_eq!(reason(&first), Some("ambiguous target endpoints"));
    items
        .get_mut("AWS::EC2::VPC")
        .ok_or("missing target")?
        .reverse();
    let second = remap::build(&catalogs("/VpcId")?, &kinds()?, &items)?;
    assert_eq!(second.report, first.report);
    Ok(())
}
