use super::item_records;
use crate::execution::test_catalog;
use aws_wire::{
    request::Protocol,
    response::{DecodeLimits, ResponseContext, decode},
};
use catalog_tool::smithy::Registry;
use http::{HeaderMap, StatusCode};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    record::{Record, ValueRef},
};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Path = &'static [&'static str];

const SERVICE: &str = "organizations";
const DATE: &str = "2016-11-28";
const ADDRESS: &str = "owner@example.test";

struct Stored {
    filtered: Vec<Record>,
    unfiltered: Vec<Record>,
    fields: Registry,
}

fn stored(operation: &str, companions: &[&str], body: &Value) -> TestResult<Stored> {
    let mut operations = vec![operation];
    operations.extend_from_slice(companions);
    let bytes = test_catalog::compile(SERVICE, DATE, &operations, None, |_, _| Ok(()))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(SERVICE, operation)
        .ok_or("Organizations plan absent")?;
    let payload = serde_json::to_vec(body)?;
    let rows = |limits: DecodeLimits| -> TestResult<Vec<Record>> {
        let response = decode(ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol: Protocol::AwsJson11,
            body: &payload,
            headers: &HeaderMap::new(),
            status: StatusCode::OK,
            limits,
        })?;
        Ok(item_records(&response, plan.items())?)
    };
    let filtered = rows(DecodeLimits::default().excluding_members(plan.excluded_members()))?;
    let unfiltered = rows(DecodeLimits::default())?;
    let fields = Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    Ok(Stored {
        filtered,
        unfiltered,
        fields,
    })
}

fn present(row: &Record, fields: &Registry, path: &[&str]) -> TestResult<bool> {
    let Some((last, parents)) = path.split_last() else {
        return Err("empty member path".into());
    };
    let mut map = row.root();
    for name in parents {
        let id = fields.id(name).ok_or("parent field absent")?;
        match map.get_field(id)? {
            Some(ValueRef::Map(inner)) => map = inner,
            _ => return Ok(false),
        }
    }
    let id = fields.id(last).ok_or("member field absent")?;
    Ok(map.get_field(id)?.is_some())
}

fn assert_email_dropped(
    stored: &Stored,
    rows: usize,
    removed: &[Path],
    kept: &[Path],
) -> TestResult {
    assert_eq!(stored.filtered.len(), rows);
    assert_eq!(stored.unfiltered.len(), rows);
    for row in &stored.unfiltered {
        for path in removed.iter().chain(kept) {
            assert!(
                present(row, &stored.fields, path)?,
                "fixture lacks {path:?}"
            );
        }
    }
    for row in &stored.filtered {
        for path in removed {
            assert!(!present(row, &stored.fields, path)?, "{path:?} was stored");
        }
        for path in kept {
            assert!(present(row, &stored.fields, path)?, "{path:?} was lost");
        }
    }
    Ok(())
}

fn account(index: u32) -> Value {
    json!({
        "Id": format!("11112222333{index}"),
        "Arn": format!("arn:aws:organizations::111122223333:account/o-example/11112222333{index}"),
        "Email": ADDRESS,
        "Name": format!("member-{index}"),
        "Status": "ACTIVE",
        "State": "ACTIVE",
        "JoinedMethod": "CREATED",
        "JoinedTimestamp": 1_700_000_000.0
    })
}

const ACCOUNT_KEPT: &[Path] = &[&["Id"], &["Arn"], &["Name"], &["Status"], &["State"]];

#[test]
fn given_describe_organization_when_stored_then_management_email_is_dropped() -> TestResult {
    let body = json!({"Organization": {
        "Id": "o-example",
        "Arn": "arn:aws:organizations::111122223333:organization/o-example",
        "FeatureSet": "ALL",
        "MasterAccountArn": "arn:aws:organizations::111122223333:account/o-example/111122223333",
        "MasterAccountId": "111122223333",
        "MasterAccountEmail": ADDRESS,
        "AvailablePolicyTypes": [{"Type": "SERVICE_CONTROL_POLICY", "Status": "ENABLED"}]
    }});
    let stored = stored("DescribeOrganization", &[], &body)?;
    assert_email_dropped(
        &stored,
        1,
        &[&["MasterAccountEmail"]],
        &[
            &["Id"],
            &["Arn"],
            &["FeatureSet"],
            &["MasterAccountArn"],
            &["MasterAccountId"],
            &["AvailablePolicyTypes"],
        ],
    )
}

#[test]
fn given_list_accounts_when_stored_then_every_account_email_is_dropped() -> TestResult {
    let body = json!({"Accounts": [account(1), account(2)]});
    let stored = stored("ListAccounts", &[], &body)?;
    assert_email_dropped(&stored, 2, &[&["Email"]], ACCOUNT_KEPT)
}

#[test]
fn given_describe_account_when_stored_then_account_email_is_dropped() -> TestResult {
    let body = json!({"Account": account(1)});
    let stored = stored("DescribeAccount", &["ListAccounts"], &body)?;
    assert_email_dropped(&stored, 1, &[&["Email"]], ACCOUNT_KEPT)
}

#[test]
fn given_delegated_administrators_when_stored_then_email_is_dropped() -> TestResult {
    let body = json!({"DelegatedAdministrators": [{
        "Id": "111122223331",
        "Arn": "arn:aws:organizations::111122223333:account/o-example/111122223331",
        "Email": ADDRESS,
        "Name": "member-1",
        "Status": "ACTIVE",
        "State": "ACTIVE",
        "JoinedMethod": "CREATED",
        "JoinedTimestamp": 1_700_000_000.0,
        "DelegationEnabledDate": 1_700_000_100.0
    }]});
    let stored = stored("ListDelegatedAdministrators", &[], &body)?;
    assert_email_dropped(
        &stored,
        1,
        &[&["Email"]],
        &[&["Id"], &["Arn"], &["Name"], &["DelegationEnabledDate"]],
    )
}

fn transfer() -> Value {
    json!({
        "Arn": "arn:aws:organizations::111122223333:transfer/o-example/billing/rt-example",
        "Name": "billing-transfer",
        "Id": "rt-example",
        "Type": "BILLING",
        "Status": "REQUESTED",
        "Source": {"ManagementAccountId": "111122223333", "ManagementAccountEmail": ADDRESS},
        "Target": {"ManagementAccountId": "444455556666", "ManagementAccountEmail": ADDRESS},
        "StartTimestamp": 1_700_000_000.0
    })
}

#[test]
fn given_responsibility_transfers_when_stored_then_both_participant_emails_are_dropped()
-> TestResult {
    let body = json!({"ResponsibilityTransfers": [transfer()]});
    for operation in [
        "ListInboundResponsibilityTransfers",
        "ListOutboundResponsibilityTransfers",
    ] {
        let stored = stored(operation, &[], &body)?;
        assert_email_dropped(
            &stored,
            1,
            &[
                &["Source", "ManagementAccountEmail"],
                &["Target", "ManagementAccountEmail"],
            ],
            &[
                &["Arn"],
                &["Id"],
                &["Name"],
                &["Status"],
                &["Source", "ManagementAccountId"],
                &["Target", "ManagementAccountId"],
            ],
        )?;
    }
    Ok(())
}
