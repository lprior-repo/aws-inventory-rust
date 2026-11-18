mod fixture;
mod iam;
mod iot;
mod package_assertions;
mod scenario;
mod suppressions;
use crate::sweep_runtime::execution::page::cursor;
#[path = "../execution/routing.rs"]
mod routing;

use super::*;
use crate::sweep_runtime::input;
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanScope, RuntimeCatalog},
    inventory::Region,
    record::Value,
};
use scenario::{Scenario, request_contains, xml};
use serde_json::{Value as Json, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn given_metadata_list_plans_when_compiled_then_no_body_credential_paid_or_mutating_plan_is_callable()
-> TestResult {
    let catalog = fixture::catalog()?;
    for (service, operations) in [
        (
            "iot",
            &[
                "GetJobDocument",
                "DescribeJobTemplate",
                "DescribeProvisioningTemplate",
                "DescribeCertificate",
                "ListThings",
                "DescribeEndpoint",
            ][..],
        ),
        (
            "iam",
            &[
                "GetGroupPolicy",
                "GetRolePolicy",
                "GetUserPolicy",
                "GetPolicyVersion",
                "CreateServiceSpecificCredential",
                "ResetServiceSpecificCredential",
            ][..],
        ),
        ("kms", &["GetKeyPolicy", "DescribeKey"][..]),
    ] {
        for operation in operations {
            assert!(
                catalog.lookup(service, operation).is_none(),
                "forbidden call {service}.{operation} became callable"
            );
        }
    }
    Ok(())
}
