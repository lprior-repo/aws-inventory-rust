//! Unit tests for SDK callsite resolution.

use crate::HarnessError;

use super::{calls_in_source, calls_in_source_with_hint};

fn first_service(calls: &[super::CallSite]) -> Result<&str, HarnessError> {
    calls
        .first()
        .map(|call| call.sdk_service.as_str())
        .ok_or_else(|| HarnessError::invalid("expected one resolved callsite"))
}

#[test]
fn ancestor_hint_resolves_a_glob_imported_client() -> Result<(), HarnessError> {
    let source = "fn read(client: &Client) { client.list_topics().send(); }";
    let (calls, _) = calls_in_source_with_hint("crates/aws/src/sns/topic.rs", source, Some("sns"))?;
    assert_eq!(calls.len(), 1);
    assert_eq!(first_service(&calls)?, "sns");
    Ok(())
}

#[test]
fn a_single_module_import_resolves_a_prelude_client() -> Result<(), HarnessError> {
    let source = "use aws_sdk_ec2::operation::describe_instances::DescribeInstancesError;\n\
                  fn read(client: &Client) { client.describe_instances().send(); }";
    let (calls, _) = calls_in_source("crates/aws/src/ec2/collection.rs", source)?;
    assert_eq!(calls.len(), 1);
    assert_eq!(first_service(&calls)?, "ec2");
    Ok(())
}
