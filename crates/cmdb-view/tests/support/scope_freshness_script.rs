use cmdb_view::ScopeOutcome;
use inventory_core::inventory::{
    AccountId, Kind, OperationId, Region, ResourceId, ResourceKey, ScopeKey, ServiceId,
};
use std::collections::BTreeMap;
use std::error::Error;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub const KIND: &str = "AWS::Example::Thing";
const LONG_OPERATION: &str = "ListEveryThingThatThisAccountHasEverCreatedInAnyAvailabilityZone";

type Key = (&'static str, &'static str, &'static str, &'static str, bool);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expected {
    pub account: &'static str,
    pub region: &'static str,
    pub service: &'static str,
    pub operation: &'static str,
    pub has_parent: bool,
    pub sweep_id: &'static str,
    pub outcome: ScopeOutcome,
    pub observed_at_ms: i64,
    pub last_complete_at_ms: Option<i64>,
}

impl Expected {
    fn order_key(&self) -> (i64, &'static str, &'static str, &'static str, &'static str) {
        (
            self.observed_at_ms,
            self.account,
            self.region,
            self.service,
            self.operation,
        )
    }
}

pub struct Attempt {
    pub sweep: &'static str,
    pub account: &'static str,
    pub region: &'static str,
    pub service: &'static str,
    pub operation: &'static str,
    pub parent: bool,
    pub outcome: ScopeOutcome,
    pub at: i64,
}

impl Attempt {
    fn key(&self) -> Key {
        (
            self.account,
            self.region,
            self.service,
            self.operation,
            self.parent,
        )
    }
}

const fn attempt(
    sweep: &'static str,
    (account, region, service, operation): (&'static str, &'static str, &'static str, &'static str),
    parent: bool,
    outcome: ScopeOutcome,
    at: i64,
) -> Attempt {
    Attempt {
        sweep,
        account,
        region,
        service,
        operation,
        parent,
        outcome,
        at,
    }
}

const EAST: (&str, &str, &str, &str) = ("111111111111", "us-east-1", "example", "ListThings");
const WEST: (&str, &str, &str, &str) = ("111111111111", "us-west-2", "example", "ListThings");
const OTHER_ACCOUNT: (&str, &str, &str, &str) =
    ("222222222222", "us-east-1", "example", "ListThings");
const GLOBAL: (&str, &str, &str, &str) = ("111111111111", "global", "iam", "ListRoles");
const LONG: (&str, &str, &str, &str) = (
    "111111111111",
    "eu-central-1",
    "elasticloadbalancing",
    LONG_OPERATION,
);
const CHILD: (&str, &str, &str, &str) = ("111111111111", "us-east-1", "example", "ListThings");

pub fn script() -> Vec<Attempt> {
    vec![
        attempt("sweep-1", EAST, false, ScopeOutcome::Complete, 1_000),
        attempt("sweep-1", WEST, false, ScopeOutcome::Partial, 1_100),
        attempt("sweep-1", OTHER_ACCOUNT, false, ScopeOutcome::Failed, 1_200),
        attempt("sweep-1", GLOBAL, false, ScopeOutcome::Inaccessible, 1_300),
        attempt("sweep-1", LONG, false, ScopeOutcome::Unsupported, 1_400),
        attempt("sweep-1", CHILD, true, ScopeOutcome::Complete, 1_500),
        attempt(
            "sweep-1",
            ("111111111111", "ca-central-1", "example", "ListThings"),
            false,
            ScopeOutcome::NotApplicable,
            1_600,
        ),
        attempt("sweep-2", EAST, false, ScopeOutcome::Partial, 5_000),
        attempt("sweep-2", WEST, false, ScopeOutcome::Complete, 5_100),
        attempt("sweep-2", OTHER_ACCOUNT, false, ScopeOutcome::Failed, 5_200),
    ]
}

pub fn ordered_script() -> Vec<Attempt> {
    let mut ordered = script();
    ordered.sort_by_key(|attempt| attempt.at);
    ordered
}

pub fn expected() -> Vec<Expected> {
    let mut by_scope: BTreeMap<Key, Expected> = BTreeMap::new();
    for attempt in ordered_script() {
        let previous = by_scope
            .get(&attempt.key())
            .and_then(|row| row.last_complete_at_ms);
        let last_complete_at_ms = if attempt.outcome == ScopeOutcome::Complete {
            Some(attempt.at)
        } else {
            previous
        };
        by_scope.insert(
            attempt.key(),
            Expected {
                account: attempt.account,
                region: attempt.region,
                service: attempt.service,
                operation: attempt.operation,
                has_parent: attempt.parent,
                sweep_id: attempt.sweep,
                outcome: attempt.outcome,
                observed_at_ms: attempt.at,
                last_complete_at_ms,
            },
        );
    }
    let mut rows: Vec<Expected> = by_scope.into_values().collect();
    rows.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
    rows
}

pub fn modeled(attempt: &Attempt) -> bool {
    attempt.service == "example" && attempt.operation == "ListThings"
}

fn parent_resource(account: &str, region: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        kind: Kind::parse(KIND)?,
        id: ResourceId::parse("parent-1")?,
    })
}

pub fn scope(attempt: &Attempt) -> TestResult<ScopeKey> {
    let parent = if attempt.parent {
        Some(parent_resource(attempt.account, attempt.region)?)
    } else {
        None
    };
    Ok(ScopeKey::new(
        AccountId::parse(attempt.account)?,
        Region::parse(attempt.region)?,
        ServiceId::parse(attempt.service)?,
        OperationId::parse(attempt.operation)?,
        parent,
    )?)
}

pub fn raw_scope_key(attempt: &Attempt) -> TestResult<Vec<u8>> {
    let mut key = Vec::new();
    for text in [
        attempt.account,
        attempt.region,
        attempt.service,
        attempt.operation,
    ] {
        key.extend_from_slice(&u32::try_from(text.len())?.to_be_bytes());
        key.extend_from_slice(text.as_bytes());
    }
    key.push(u8::from(attempt.parent));
    Ok(key)
}
