use super::*;
use crate::e2e::verify::coverage_status::Plans;
use std::collections::{BTreeMap, BTreeSet};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_persisted_failure_details_when_read_then_only_valid_codes_are_reported() {
    let long = format!("AWS service failure: {}", "x".repeat(129));
    for (detail, code) in [
        (
            Some("AWS service failure: AccessDenied (SecretAccessDenied)"),
            "SecretAccessDenied",
        ),
        (Some("AWS service failure: AccessDenied"), "AccessDenied"),
        (Some("Prefix: x"), "Prefix"),
        (None, "unknown"),
        (Some(""), "unknown"),
        (Some("AWS service failure: dénié"), "unknown"),
        (Some(long.as_str()), "unknown"),
    ] {
        assert_eq!(failed_code(detail), code, "{detail:?}");
    }
}

#[test]
fn given_seed_matrix_file_when_reported_then_rejection_is_joined_and_unknown_steps_fail()
-> TestResult {
    let work = tempfile::tempdir()?;
    let matrix = work.path().join("seed-matrix.tsv");
    let report = work.path().join("coverage-status.report");
    let step = "test\tCreateThing\tthing";
    let kind = "AWS::Test::Thing";
    let coverage = Coverage {
        callable: Plans::from([(kind.into(), BTreeSet::from(["test:ListThings".into()]))]),
        deferred: BTreeMap::new(),
        steps: BTreeMap::from([(step.into(), kind.into())]),
    };
    let mut evidence = Evidence::default();
    std::fs::write(&matrix, format!("{step}\trejected\tBadRequest\tdetail\n"))?;
    coverage.rejected(&read_text(&matrix)?, &mut evidence)?;
    coverage_status::publish(&report, &coverage.callable, &evidence, Selection::default())?;
    assert_eq!(
        std::fs::read_to_string(&report)?,
        concat!(
            "totals\tcallable=1\temulator_verified=0\timplemented=1\n",
            "AWS::Test::Thing\timplemented\tseed rejected: BadRequest\n",
        )
    );
    std::fs::write(
        &matrix,
        "test\tCreateThing\tunknown\trejected\tBadRequest\tdetail\n",
    )?;
    let error = coverage
        .rejected(&read_text(&matrix)?, &mut evidence)
        .err()
        .ok_or("unknown rejection step passed")?;
    assert_eq!(
        error.to_string(),
        "rejected seed matrix step is not in recipes"
    );
    Ok(())
}

fn scope_key(operation: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut key = Vec::new();
    for text in ["123456789012", "us-east-1", "test", operation] {
        key.extend_from_slice(&u32::try_from(text.len())?.to_be_bytes());
        key.extend_from_slice(text.as_bytes());
    }
    key.push(0);
    Ok(key)
}

#[tokio::test]
#[ignore = "requires an owned PostgreSQL server via INVENTORY_POSTGRES_URL"]
async fn given_not_applicable_outcomes_when_reported_then_only_a_sole_prerequisite_is_not_applicable()
-> TestResult {
    let database = crate::postgres::TestDatabase::create(&crate::postgres::admin_url()?).await?;
    let mut connection = PgConnection::connect(database.url()).await?;
    sqlx::raw_sql(
        "CREATE SCHEMA inventory; \
         CREATE TABLE inventory.sweeps(sweep_id text PRIMARY KEY,account text NOT NULL,partition text NOT NULL DEFAULT 'aws',started_at bigint NOT NULL,ended_at bigint NOT NULL DEFAULT 0,completed bigint NOT NULL DEFAULT 0); \
         CREATE TABLE inventory.scope_outcomes(sweep_id text NOT NULL,scope_key bytea NOT NULL,outcome text NOT NULL,observed_at bigint NOT NULL,detail text CHECK(detail IS NULL OR octet_length(detail)<=1024),PRIMARY KEY(sweep_id,scope_key)); \
         INSERT INTO inventory.sweeps(sweep_id,account,started_at,completed) VALUES('sweep','123456789012',1,1)",
    )
    .execute(&mut connection)
    .await?;
    for (operation, outcome) in [
        ("test:ListComplete", "complete"),
        ("test:ListNotApplicable", "not_applicable"),
        ("test:ListPrerequisite", "not_applicable"),
    ] {
        let key = scope_key(operation.split_once(':').ok_or("operation")?.1)?;
        sqlx::query(
            "INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) VALUES('sweep',$1,$2,0,NULL)",
        )
        .bind(key)
        .bind(outcome)
        .execute(&mut connection)
        .await?;
    }
    connection.close().await?;
    let work = tempfile::tempdir()?;
    std::fs::write(work.path().join("seed-matrix.tsv"), "")?;
    let coverage = Coverage {
        callable: Plans::from([
            ("AWS::Test::Empty".into(), BTreeSet::from(["test:ListEmpty".into()])),
            (
                "AWS::Test::Mixed".into(),
                BTreeSet::from([
                    "test:ListComplete".into(),
                    "test:ListNotApplicable".into(),
                ]),
            ),
            (
                "AWS::Test::Prerequisite".into(),
                BTreeSet::from(["test:ListPrerequisite".into()]),
            ),
        ]),
        deferred: BTreeMap::new(),
        steps: BTreeMap::new(),
    };
    coverage
        .report(database.url(), work.path(), &Keys::new(), Selection::default())
        .await?;
    assert_eq!(
        std::fs::read_to_string(work.path().join("coverage-status.report"))?,
        concat!(
            "totals\tcallable=3\temulator_verified=0\timplemented=3\n",
            "AWS::Test::Empty\timplemented\tempty: nothing seeded\n",
            "AWS::Test::Mixed\timplemented\tempty: nothing seeded\n",
            "AWS::Test::Prerequisite\timplemented\tnot applicable: prerequisite disabled\n",
        )
    );
    Ok(())
}
