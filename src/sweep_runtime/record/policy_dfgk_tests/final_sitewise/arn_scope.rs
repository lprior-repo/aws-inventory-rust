use super::*;
use crate::sweep_runtime::{SweepError, record};
use inventory_core::{
    catalog::runtime::IdentityPlan,
    inventory::{AccountId, RegionId},
    record::{Key, Record, Value},
};
use std::collections::BTreeMap;

#[test]
fn given_summary_arn_identities_when_returned_scope_is_malformed_or_foreign_then_every_new_arn_producer_fails_closed()
-> TestResult {
    // Given actual compiled identity plans, not locally reimplemented ARN parsing.
    let catalog = catalog()?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    for (service, operation, resource) in [
        ("iotsitewise", "ListWorkspaces", "workspace/factory"),
        (
            "iotsitewise",
            "ListApplications",
            "application/11111111-1111-4111-8111-111111111111",
        ),
        (
            "iotsitewise",
            "ListDatasets",
            "dataset/11111111-1111-4111-8111-111111111111",
        ),
        (
            "iotsitewise",
            "ListPipelines",
            "workspace/factory/pipeline/definition",
        ),
        (
            "iotsitewise",
            "ListTasks",
            "workspace/factory/task/definition",
        ),
        (
            "iotsitewise",
            "ListTimeSeries",
            "time-series/11111111-1111-4111-8111-111111111111",
        ),
        (
            "iottwinmaker",
            "ListScenes",
            "workspace/factory/scene/shared",
        ),
        (
            "iottwinmaker",
            "ListSyncJobs",
            "workspace/factory/sync-job/SITEWISE",
        ),
    ] {
        let plan = plan(&catalog, service, operation)?;
        let IdentityPlan::Arn(path) = plan.identity() else {
            return Err("summary lost strict ARN identity".into());
        };
        assert_eq!(path.len(), 1);
        let field = *path.first().ok_or("ARN identity path absent")?;
        let row = |value: &str| -> TestResult<Record> {
            Ok(Record::encode(&Value::Map(BTreeMap::from([(
                Key::Field(field),
                Value::Text(value.to_owned()),
            )])))?)
        };
        let valid = arn(service, resource);
        // When resolving the modeled identity with the real request owner scope.
        assert_eq!(
            record::resource_id(
                &row(&valid)?,
                plan.identity(),
                Some(record::IdentityContext {
                    account: &account,
                    region: &region
                })
            )?
            .as_str(),
            valid
        );
        for invalid in [
            "not-an-arn".to_owned(),
            format!("arn:aws:{service}:us-east-1:invalid:{resource}"),
            format!("arn:aws:{service}:us-east-1:123456789012:"),
        ] {
            assert_eq!(
                matches!(
                    record::resource_id(
                        &row(&invalid)?,
                        plan.identity(),
                        Some(record::IdentityContext {
                            account: &account,
                            region: &region
                        })
                    ),
                    Err(SweepError::Operation("modeled ARN is malformed"))
                ),
                true,
                "{service}.{operation}"
            );
        }
        // Then foreign account/region/partition are rejected, never silently re-owned locally.
        for invalid in [
            format!("arn:aws:{service}:us-east-1:999999999999:{resource}"),
            format!("arn:aws:{service}:us-west-2:123456789012:{resource}"),
            format!("arn:aws-cn:{service}:us-east-1:123456789012:{resource}"),
        ] {
            assert_eq!(
                matches!(
                    record::resource_id(
                        &row(&invalid)?,
                        plan.identity(),
                        Some(record::IdentityContext {
                            account: &account,
                            region: &region
                        })
                    ),
                    Err(SweepError::Operation(
                        "modeled ARN does not match trusted request scope"
                    ))
                ),
                true,
                "{service}.{operation}"
            );
        }
    }
    Ok(())
}
