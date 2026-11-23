use inventory_core::record::{Key, Value};
use inventory_harness::inventory_seed::{
    FieldPath, ScenarioSeedOracle, SeedError, SeedIdentityEvidence, SeedInput, SeedInputValue,
    SeedOutputCheck, SeedScenario, SeedScope, SeedStep, StepId,
};
use std::{collections::BTreeMap, error::Error};

type TestResult = Result<(), Box<dyn Error>>;

// Given fresh local emulator state, these recipes create metadata only. The
// common entrypoint executes them and compares independent seed evidence with
// collected resources. No sends, confirmation delivery, polling or executions.
pub(super) fn sqs_scenario() -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![
        queue_create(0, "inventory-seed-sqs-standard-a", false)?,
        queue_read(1, 0)?,
        queue_create(2, "inventory-seed-sqs-standard-b", false)?,
        queue_read(3, 2)?,
        queue_create(4, "inventory-seed-sqs-fifo.fifo", true)?,
        queue_read(5, 4)?,
    ])
}

pub(super) fn sns_scenario() -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![
        topic(0, "inventory-seed-sns-standard-a", false)?,
        topic(1, "inventory-seed-sns-standard-b", false)?,
        topic(2, "inventory-seed-sns-fifo.fifo", true)?,
        queue_create(3, "inventory-seed-sns-endpoint", false)?,
        queue_read(4, 3)?,
        subscribe()?,
        subscription()?,
    ])
}

pub(super) fn sfn_scenario() -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![
        activity(0, "inventory-seed-sfn-activity-a")?,
        activity(1, "inventory-seed-sfn-activity-b")?,
        execution_role()?,
        state_machine(3, "inventory-seed-sfn-standard", "STANDARD")?,
        state_machine(4, "inventory-seed-sfn-express", "EXPRESS")?,
    ])
}

fn text(field: &str, value: &str) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(
        FieldPath::new(&[field])?,
        SeedInputValue::Text(value.into()),
    ))
}

fn previous(field: &str, step: u16, path: &[&str]) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(
        FieldPath::new(&[field])?,
        SeedInputValue::PreviousOutput {
            step: StepId::new(step),
            path: FieldPath::new(path)?,
        },
    ))
}

fn attributes(entries: &[(&str, &str)]) -> Result<SeedInput, SeedError> {
    let values = entries
        .iter()
        .map(|(key, value)| (Key::Name((*key).into()), Value::Text((*value).into())))
        .collect::<BTreeMap<_, _>>();
    Ok(SeedInput::new(
        FieldPath::new(&["Attributes"])?,
        SeedInputValue::Literal(Value::Map(values)),
    ))
}

fn queue_create(id: u16, name: &str, fifo: bool) -> Result<SeedStep, SeedError> {
    let inputs = if fifo {
        vec![
            text("QueueName", name)?,
            attributes(&[("FifoQueue", "true"), ("ContentBasedDeduplication", "true")])?,
        ]
    } else {
        vec![text("QueueName", name)?]
    };
    Ok(SeedStep::auxiliary(
        StepId::new(id),
        "sqs".into(),
        "CreateQueue".into(),
        inputs,
    ))
}

fn queue_read(id: u16, create: u16) -> Result<SeedStep, SeedError> {
    // QueueUrl is transport data, not the canonical queue oracle identity.
    Ok(SeedStep::new(
        StepId::new(id),
        "sqs".into(),
        "GetQueueAttributes".into(),
        "AWS::SQS::Queue".into(),
        FieldPath::new(&["Attributes", "QueueArn"])?,
        vec![
            previous("QueueUrl", create, &["QueueUrl"])?,
            SeedInput::new(
                FieldPath::new(&["AttributeNames"])?,
                SeedInputValue::Literal(Value::List(vec![Value::Text("QueueArn".into())])),
            ),
        ],
    ))
}

fn topic(id: u16, name: &str, fifo: bool) -> Result<SeedStep, SeedError> {
    let inputs = if fifo {
        vec![
            text("Name", name)?,
            attributes(&[("FifoTopic", "true"), ("ContentBasedDeduplication", "true")])?,
        ]
    } else {
        vec![text("Name", name)?]
    };
    Ok(SeedStep::new(
        StepId::new(id),
        "sns".into(),
        "CreateTopic".into(),
        "AWS::SNS::Topic".into(),
        FieldPath::new(&["TopicArn"])?,
        inputs,
    ))
}

fn subscribe() -> Result<SeedStep, SeedError> {
    // Same-account SQS endpoints need no confirmation or external delivery.
    Ok(SeedStep::auxiliary(
        StepId::new(5),
        "sns".into(),
        "Subscribe".into(),
        vec![
            previous("TopicArn", 0, &["TopicArn"])?,
            text("Protocol", "sqs")?,
            previous("Endpoint", 4, &["Attributes", "QueueArn"])?,
            SeedInput::new(
                FieldPath::new(&["ReturnSubscriptionArn"])?,
                SeedInputValue::Boolean(true),
            ),
        ],
    ))
}

fn subscription() -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(6),
        "sns".into(),
        "GetSubscriptionAttributes".into(),
        "AWS::SNS::Subscription".into(),
        FieldPath::new(&["Attributes", "SubscriptionArn"])?,
        vec![previous("SubscriptionArn", 5, &["SubscriptionArn"])?],
    )
    .with_response_checks(vec![SeedOutputCheck::new(
        FieldPath::new(&["Attributes", "SubscriptionArn"])?,
        FieldPath::new(&["SubscriptionArn"])?,
    )]))
}

fn activity(id: u16, name: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id),
        "sfn".into(),
        "CreateActivity".into(),
        "AWS::StepFunctions::Activity".into(),
        FieldPath::new(&["activityArn"])?,
        vec![text("name", name)?],
    ))
}

fn execution_role() -> Result<SeedStep, SeedError> {
    let trust = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"states.amazonaws.com"},"Action":"sts:AssumeRole"}]}"#;
    Ok(SeedStep::new(
        StepId::new(2),
        "iam".into(),
        "CreateRole".into(),
        "AWS::IAM::Role".into(),
        FieldPath::new(&["Role", "Arn"])?,
        vec![
            text("RoleName", "inventory-seed-sfn-execution-role")?,
            text("AssumeRolePolicyDocument", trust)?,
        ],
    )
    .with_scope(SeedScope::Account)
    .with_response_checks(vec![SeedOutputCheck::new(
        FieldPath::new(&["Role", "RoleName"])?,
        FieldPath::new(&["RoleName"])?,
    )]))
}

fn state_machine(id: u16, name: &str, machine_type: &str) -> Result<SeedStep, SeedError> {
    // Generated fixture ASL contains no customer data or service integrations.
    let definition = r#"{"StartAt":"Complete","States":{"Complete":{"Type":"Succeed"}}}"#;
    Ok(SeedStep::new(
        StepId::new(id),
        "sfn".into(),
        "CreateStateMachine".into(),
        "AWS::StepFunctions::StateMachine".into(),
        FieldPath::new(&["stateMachineArn"])?,
        vec![
            text("name", name)?,
            text("definition", definition)?,
            previous("roleArn", 2, &["Role", "Arn"])?,
            text("type", machine_type)?,
        ],
    ))
}

// Then: exact fixture names, canonical identities, steps, scopes and response
// provenance must survive. Counts also reject helper records leaking as resources.
pub(super) fn assert_sqs_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> TestResult {
    assert_resources(
        oracle,
        account,
        region,
        &[
            (
                1,
                "AWS::SQS::Queue",
                format!("arn:aws:sqs:{region}:{account}:inventory-seed-sqs-standard-a"),
                region,
            ),
            (
                3,
                "AWS::SQS::Queue",
                format!("arn:aws:sqs:{region}:{account}:inventory-seed-sqs-standard-b"),
                region,
            ),
            (
                5,
                "AWS::SQS::Queue",
                format!("arn:aws:sqs:{region}:{account}:inventory-seed-sqs-fifo.fifo"),
                region,
            ),
        ],
    );
    Ok(())
}

pub(super) fn assert_sns_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> TestResult {
    let subscription = oracle
        .resources
        .get(4)
        .ok_or("missing subscription resource")?;
    let prefix = format!("arn:aws:sns:{region}:{account}:inventory-seed-sns-standard-a:");
    let suffix = subscription
        .id
        .as_str()
        .strip_prefix(prefix.as_str())
        .ok_or("subscription ARN has wrong topic/scope")?;
    if suffix.is_empty() || suffix.contains(':') {
        return Err("subscription ARN has invalid opaque identifier".into());
    }
    // Its generated suffix is seed evidence, never invented or collector-derived.
    // The read's response check separately matches Subscribe's returned exact ARN.
    assert_resources(
        oracle,
        account,
        region,
        &[
            (
                0,
                "AWS::SNS::Topic",
                format!("arn:aws:sns:{region}:{account}:inventory-seed-sns-standard-a"),
                region,
            ),
            (
                1,
                "AWS::SNS::Topic",
                format!("arn:aws:sns:{region}:{account}:inventory-seed-sns-standard-b"),
                region,
            ),
            (
                2,
                "AWS::SNS::Topic",
                format!("arn:aws:sns:{region}:{account}:inventory-seed-sns-fifo.fifo"),
                region,
            ),
            (
                4,
                "AWS::SQS::Queue",
                format!("arn:aws:sqs:{region}:{account}:inventory-seed-sns-endpoint"),
                region,
            ),
            (
                6,
                "AWS::SNS::Subscription",
                subscription.id.to_string(),
                region,
            ),
        ],
    );
    Ok(())
}

pub(super) fn assert_sfn_oracle(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
) -> TestResult {
    assert_resources(
        oracle,
        account,
        region,
        &[
            (
                0,
                "AWS::StepFunctions::Activity",
                format!("arn:aws:states:{region}:{account}:activity:inventory-seed-sfn-activity-a"),
                region,
            ),
            (
                1,
                "AWS::StepFunctions::Activity",
                format!("arn:aws:states:{region}:{account}:activity:inventory-seed-sfn-activity-b"),
                region,
            ),
            (
                2,
                "AWS::IAM::Role",
                format!("arn:aws:iam::{account}:role/inventory-seed-sfn-execution-role"),
                "global",
            ),
            (
                3,
                "AWS::StepFunctions::StateMachine",
                format!(
                    "arn:aws:states:{region}:{account}:stateMachine:inventory-seed-sfn-standard"
                ),
                region,
            ),
            (
                4,
                "AWS::StepFunctions::StateMachine",
                format!(
                    "arn:aws:states:{region}:{account}:stateMachine:inventory-seed-sfn-express"
                ),
                region,
            ),
        ],
    );
    Ok(())
}

fn assert_resources(
    oracle: &ScenarioSeedOracle,
    account: &str,
    region: &str,
    expected: &[(u16, &str, String, &str)],
) {
    assert_eq!(oracle.account_id.as_str(), account);
    assert_eq!(oracle.region.as_str(), region);
    assert_eq!(oracle.resources.len(), expected.len());
    // No relationship is fabricated: SNS has no modeled relation plan, and
    // CreateStateMachine omits roleArn from the independently observed response.
    assert_eq!(oracle.edges.len(), 0);
    oracle
        .resources
        .iter()
        .zip(expected)
        .for_each(|(resource, (step, kind, id, scope))| {
            assert_eq!(resource.step, StepId::new(*step));
            assert_eq!(resource.kind.as_str(), *kind);
            assert_eq!(resource.id.as_str(), id.as_str());
            assert_eq!(resource.evidence, SeedIdentityEvidence::Response);
            assert_eq!(resource.key.account.as_str(), account);
            assert_eq!(resource.key.region.as_str(), *scope);
            assert_eq!(resource.key.kind.as_str(), *kind);
            assert_eq!(resource.key.id.as_str(), id.as_str());
        });
}
