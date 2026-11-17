#[path = "../../../crates/catalog-tool/tests/support/identity.rs"]
mod fixture;

use super::*;
use fixture::{TestResult, compiled};
use inventory_core::{catalog::runtime::RuntimeCatalog, record::{Key, Value}};
use catalog_tool::overlay::{Identity, MemberPath};
use std::collections::BTreeMap;

fn row() -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(i64::MAX)),
        (Key::Field(FieldId::new(2)?), Value::Bool(false)),
        (Key::Field(FieldId::new(6)?), Value::Text("active-wire".to_owned())),
        (Key::Field(FieldId::new(90)?), Value::Text("000000000000".to_owned())),
        (Key::Field(FieldId::new(91)?), Value::Text("cn-north-1".to_owned())),
    ])))?)
}

#[test]
fn given_compiled_s3_template_when_extracted_then_bucket_arn_uses_commercial_context() -> TestResult {
    let bytes = compiled(Identity::Template("arn:{partition}:s3:::{Name}".to_owned()), "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-west-2")?;
    let id = resource_id(&row()?, plan.identity(), Some(IdentityContext { account: &account, region: &region }))?;
    assert_eq!(id.as_str(), "arn:aws:s3:::active-wire");
    Ok(())
}

#[test]
fn given_compiled_context_template_when_extracted_then_request_scope_overrides_record_spoofs() -> TestResult {
    let bytes = compiled(Identity::Template("arn:{partition}:things:{region}:{account}:{Name}/{Count}/{Flag}".to_owned()), "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("eu-central-1")?;
    let id = resource_id(&row()?, plan.identity(), Some(IdentityContext { account: &account, region: &region }))?;
    assert_eq!(id.as_str(), "arn:aws:things:eu-central-1:123456789012:active-wire/9223372036854775807/false");
    assert!(resource_id(&row()?, plan.identity(), None).is_err());
    Ok(())
}

#[test]
fn given_compiled_template_when_context_is_noncommercial_then_extraction_fails_closed() -> TestResult {
    let bytes = compiled(Identity::Template("arn:{partition}:things:{region}:{account}:{Name}".to_owned()), "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let account = AccountId::parse("123456789012")?;
    ["cn-north-1", "us-gov-west-1", "us-iso-east-1", "eu-isoe-west-1", "eusc-de-east-1", "unknown-place-1"].into_iter().try_for_each(|region| {
        let region = RegionId::parse(region)?;
        assert!(resource_id(&row()?, plan.identity(), Some(IdentityContext { account: &account, region: &region })).is_err());
        Ok(())
    })
}

#[test]
fn given_compiled_scalar_template_when_extracted_then_selected_value_matches_describe_arn() -> TestResult {
    let bytes = compiled(Identity::Template("arn:{partition}:dynamodb:{region}:{account}:table/{$}".to_owned()), "active-wire", true)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let scalar = scalar_record(ValueRef::Text("active-wire"))?;
    let id = resource_id(&scalar, plan.identity(), Some(IdentityContext { account: &account, region: &region }))?;
    assert_eq!(id.as_str(), "arn:aws:dynamodb:us-east-1:123456789012:table/active-wire");
    let described = scalar_record(ValueRef::Text(id.as_str()))?;
    assert_eq!(resource_id(&described, &IdentityPlan::Arn(Box::new([])), None)?, id);
    Ok(())
}

#[test]
fn given_compiled_template_when_required_field_is_missing_or_oversized_then_extraction_fails() -> TestResult {
    let bytes = compiled(Identity::Template("bucket/{Name}".to_owned()), "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    assert!(resource_id(&empty, plan.identity(), None).is_err());
    let huge = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(6)?), Value::Text("a".repeat(4096))),
    ])))?;
    assert!(resource_id(&huge, plan.identity(), None).is_err());
    Ok(())
}

#[test]
fn given_compiled_singleton_structure_when_selected_then_one_relative_identity_is_extracted() -> TestResult {
    let bytes = fixture::compiled_singleton(fixture::key("Name")?)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let response = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(3)?), ValueRef::Map(row()?.root()).to_value()?),
    ])))?;
    let items = item_records(&response, plan.items())?;
    assert_eq!(items.len(), 1);
    assert_eq!(resource_id(items.first().ok_or("item absent")?, plan.identity(), None)?.as_str(), "active-wire");
    let null_response = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(3)?), Value::Null),
    ])))?;
    assert!(item_records(&null_response, plan.items())?.is_empty());
    Ok(())
}

#[test]
fn given_enum_member_identity_when_compiled_then_each_plan_extracts_exact_resource_text() -> TestResult {
    [
        (fixture::key("Name")?, "active-wire"),
        (Identity::Template("name/{Name}".to_owned()), "name/active-wire"),
    ].into_iter().try_for_each(|(identity, expected)| -> TestResult {
        let bytes = compiled(identity, "active-wire", false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
        assert_eq!(resource_id(&row()?, plan.identity(), None)?.as_str(), expected);
        Ok(())
    })
}

#[test]
fn given_enum_text_that_is_not_an_arn_when_arn_identity_is_extracted_then_it_fails_closed() -> TestResult {
    [("Name", "active-wire", false), ("$", "second-wire", true)].into_iter().try_for_each(|(member, wire, scalar)| -> TestResult {
        let bytes = compiled(fixture::arn(member)?, wire, scalar)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
        let record = if scalar { scalar_record(ValueRef::Text(wire))? } else { row()? };
        assert!(matches!(resource_id(&record, plan.identity(), None), Err(SweepError::Operation("modeled ARN is malformed"))));
        Ok(())
    })
}

#[test]
fn given_enum_scalar_identity_when_compiled_then_self_extraction_preserves_exact_text() -> TestResult {
    [
        (fixture::key("$")?, "second-wire"),
        (Identity::Template("prefix/{$}".to_owned()), "prefix/second-wire"),
    ].into_iter().try_for_each(|(identity, expected)| -> TestResult {
        let bytes = compiled(identity, "second-wire", true)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
        let scalar = scalar_record(ValueRef::Text("second-wire"))?;
        assert_eq!(resource_id(&scalar, plan.identity(), None)?.as_str(), expected);
        Ok(())
    })
}

#[test]
fn compiled_composite_identity_accepts_exact_bound_and_rejects_oversize_components() -> TestResult {
    let bytes = compiled(
        Identity::Key(vec![MemberPath::new("Name")?, MemberPath::new("OtherName")?]),
        "active-wire",
        false,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let encode = |first: &str, second: &str| -> TestResult<Record> {
        Ok(Record::encode(&Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(6)?), Value::Text(first.to_owned())),
            (Key::Field(FieldId::new(7)?), Value::Text(second.to_owned())),
        ])))?)
    };
    let first = "a".repeat(2042);
    let second = "b".repeat(2043);
    let accepted = resource_id(&encode(&first, &second)?, plan.identity(), None)?;
    assert_eq!(accepted.as_str(), format!("2042:{first}|2043:{second}"));
    assert_eq!(accepted.as_str().len(), 4096);
    [(2043, 2043), (1_048_576, 1), (1, 1_048_576)]
        .into_iter()
        .try_for_each(|(first, second)| -> TestResult {
            let result = resource_id(
                &encode(&"a".repeat(first), &"b".repeat(second))?,
                plan.identity(),
                None,
            );
            assert!(matches!(
                result,
                Err(SweepError::Operation("modeled resource identity is too long"))
            ));
            Ok(())
        })
}
