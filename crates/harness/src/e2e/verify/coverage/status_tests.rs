use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn catalog() -> Plans {
    BTreeMap::from([
        (
            "AWS::Test::Empty".into(),
            BTreeSet::from(["test:ListEmpty".into()]),
        ),
        (
            "AWS::Test::Failed".into(),
            BTreeSet::from(["test:ListFailed".into()]),
        ),
        (
            "AWS::Test::None".into(),
            BTreeSet::from(["test:ListNone".into()]),
        ),
        (
            "AWS::Test::Rejected".into(),
            BTreeSet::from(["test:ListRejected".into()]),
        ),
        (
            "AWS::Test::Unsupported".into(),
            BTreeSet::from(["test:ListUnsupported".into()]),
        ),
        (
            "AWS::Test::Verified".into(),
            BTreeSet::from(["test:ListVerified".into(), "test:ListOther".into()]),
        ),
    ])
}

fn evidence(reverse: bool) -> Evidence {
    let mut evidence = Evidence::default();
    let mut probes = [
        ("test:ListEmpty", Probe::Complete),
        ("test:ListFailed", Probe::Failed("ZError")),
        ("test:ListFailed", Probe::Failed("AError")),
        ("test:ListUnsupported", Probe::Unsupported),
        ("test:ListRejected", Probe::Unsupported),
        ("test:ListOther", Probe::Failed("OtherFailure")),
    ];
    if reverse {
        probes.reverse();
    }
    probes
        .into_iter()
        .for_each(|(operation, probe)| evidence.probe(operation.into(), probe));
    evidence.observe("AWS::Test::None", "other:NotACatalogPlan");
    evidence.observe("AWS::Test::Verified", "test:ListVerified");
    evidence.reject("AWS::Test::Rejected", "DeniedException");
    evidence
}

#[test]
fn given_plan_kinds_and_mixed_evidence_when_reported_then_each_kind_has_its_exact_status()
-> TestResult {
    let root = tempfile::tempdir()?;
    let file = root.path().join("coverage-status.report");
    publish(&file, &catalog(), &evidence(false), Selection::default())?;
    assert_eq!(
        std::fs::read_to_string(file)?,
        concat!(
            "totals\tcallable=6\temulator_verified=1\timplemented=5\n",
            "AWS::Test::Empty\timplemented\tempty: nothing seeded\n",
            "AWS::Test::Failed\timplemented\tfailed: AError\n",
            "AWS::Test::None\timplemented\tempty: nothing seeded\n",
            "AWS::Test::Rejected\timplemented\tseed rejected: DeniedException\n",
            "AWS::Test::Unsupported\timplemented\tunsupported\n",
            "AWS::Test::Verified\temulator-verified\ttest:ListVerified\n",
        )
    );
    Ok(())
}

#[test]
fn given_same_evidence_in_different_orders_when_regenerated_then_report_bytes_are_identical()
-> TestResult {
    let root = tempfile::tempdir()?;
    let first = root.path().join("first");
    let second = root.path().join("second");
    let plans = catalog();
    publish(&first, &plans, &evidence(false), Selection::default())?;
    publish(
        &second,
        &plans.into_iter().rev().collect(),
        &evidence(true),
        Selection::default(),
    )?;
    assert_eq!(std::fs::read(first)?, std::fs::read(second)?);
    Ok(())
}

#[test]
fn given_written_report_with_missing_and_extra_kinds_when_read_back_then_both_are_named()
-> TestResult {
    let plans = catalog();
    let root = tempfile::tempdir()?;
    let path = root.path().join("coverage-status.report");
    publish(&path, &plans, &evidence(false), Selection::default())?;
    let content = std::fs::read_to_string(&path)?;
    let cases = [
        (
            content.replace("AWS::Test::None\timplemented\tempty: nothing seeded\n", ""),
            "coverage-status kind set differs: missing=[AWS::Test::None] extra=[]",
        ),
        (
            format!("{content}AWS::Test::Extra\timplemented\tnot run\n"),
            "coverage-status kind set differs: missing=[] extra=[AWS::Test::Extra]",
        ),
    ];
    for (changed, expected) in cases {
        std::fs::write(&path, changed)?;
        let error = require_kinds(&path, &plans)
            .err()
            .ok_or("bad report accepted")?;
        assert_eq!(error.to_string(), expected);
    }
    Ok(())
}

#[test]
fn given_service_filter_when_reported_then_unselected_kinds_are_not_run() -> TestResult {
    let work = tempfile::tempdir()?;
    let output = work.path().join("coverage-status.report");
    let plans = Plans::from([
        (
            "AWS::A::Selected".into(),
            BTreeSet::from(["a:ListSelected".into()]),
        ),
        (
            "AWS::B::Prerequisite".into(),
            BTreeSet::from(["b:ListDependency".into()]),
        ),
        (
            "AWS::A::Other".into(),
            BTreeSet::from(["a:ListOther".into()]),
        ),
        (
            "AWS::C::Unselected".into(),
            BTreeSet::from(["c:ListThings".into()]),
        ),
    ]);
    publish(
        &output,
        &plans,
        &Evidence::default(),
        Selection {
            requested: &["a:ListSelected".into()],
            executed: &["a:ListSelected".into(), "b".into()],
        },
    )?;
    assert_eq!(
        std::fs::read_to_string(output)?,
        concat!(
            "totals\tcallable=4\temulator_verified=0\timplemented=4\tpartial selected=a:ListSelected\n",
            "AWS::A::Other\timplemented\tnot run\n",
            "AWS::A::Selected\timplemented\tempty: nothing seeded\n",
            "AWS::B::Prerequisite\timplemented\tempty: nothing seeded\n",
            "AWS::C::Unselected\timplemented\tnot run\n",
        )
    );
    Ok(())
}
