use catalog_tool::iam_policy::{self, Operation, render};
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type TestResult = Result<(), Box<dyn Error>>;
const REFERENCE: &str = "# Source: fixture\n# Retrieved: 2026-11-24\ns3\tListAllMyBuckets\tread\nec2\tDescribeVpcs\tread\nec2\tDescribeSubnets\tread\nec2\tDescribeSecrets\tread\nec2\tDeleteVpc\twrite\n";
const MAPPINGS: &str = "[operation]\n\"s3/ListBuckets\" = \"s3:ListAllMyBuckets\"\n";
const CLI_REFERENCE: &str = "# Retrieved: 2026-11-24\n";
const CLI_GAPS: &str = "ec2\tDescribeVpcs\tec2\tunconfirmed: absent from the service authorization reference at 2026-11-24\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn given_new_callable_operation_when_generated_then_action_appears() -> TestResult {
    let first = callable_catalog(false)?;
    let old = render(
        &iam_policy::operations(&RuntimeCatalog::from_bytes(&first)?)?,
        "",
        REFERENCE,
        "",
    )?;
    let added = callable_catalog(true)?;
    let new = render(
        &iam_policy::operations(&RuntimeCatalog::from_bytes(&added)?)?,
        "",
        REFERENCE,
        "",
    )?;
    assert!(old.actions.contains("ec2:DescribeVpcs"));
    assert!(!old.actions.contains("ec2:DescribeSubnets"));
    assert!(new.actions.contains("ec2:DescribeVpcs"));
    assert!(new.actions.contains("ec2:DescribeSubnets"));
    Ok(())
}

#[test]
fn given_missing_mapping_or_unverified_target_when_generated_then_operation_is_named() -> TestResult
{
    let ops = [Operation::new("s3", "ListBuckets", "s3")];
    let error = render(&ops, "", REFERENCE, "")
        .err()
        .ok_or("missing mapping accepted")?;
    assert!(error.to_string().contains("s3/ListBuckets"));
    let bad = "[operation]\n\"s3/ListBuckets\" = \"s3:Invented\"";
    assert!(render(&ops, bad, REFERENCE, "").is_err());
    let write = "[operation]\n\"s3/ListBuckets\" = \"ec2:DeleteVpc\"";
    assert!(render(&ops, write, REFERENCE, "").is_err());
    let empty = "[operation]\n\"s3/ListBuckets\" = \"\"";
    let empty_error = render(&ops, empty, REFERENCE, "")
        .err()
        .ok_or("empty mapping accepted")?;
    assert!(empty_error.to_string().contains("empty IAM mapping"));
    assert!(render(&ops, "[operation]\n\"s3/ListBuckets\" = []", REFERENCE, "").is_err());
    assert!(
        render(
            &ops,
            "[operation]\n\"s3/ListBuckets\" = [\"s3:ListAllMyBuckets\", \"ec2:DescribeVpcs\"]",
            REFERENCE,
            ""
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn given_reordered_operations_when_generated_then_bytes_match() -> TestResult {
    let mut ops = vec![
        Operation::new("s3", "ListBuckets", "s3"),
        Operation::new("ec2", "DescribeVpcs", "ec2"),
    ];
    let first = render(&ops, MAPPINGS, REFERENCE, "")?;
    ops.reverse();
    assert_eq!(first, render(&ops, MAPPINGS, REFERENCE, "")?);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&first.text)?
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    Ok(())
}

#[test]
fn given_generated_files_when_checking_then_byte_drift_in_either_file_fails() -> TestResult {
    let directory = fixture_root()?;
    assert!(command(directory.path(), "--write")?.status.success());
    assert!(command(directory.path(), "--check")?.status.success());
    [
        "catalog/iam-read-only-policy.json",
        "deploy/read-only-role.yaml",
        "deploy/read-only-policy-gaps.txt",
    ]
    .iter()
    .try_for_each(|path| -> TestResult {
        let file = directory.path().join(path);
        let original = fs::read(&file)?;
        let mut drift = original.clone();
        drift.push(b' ');
        fs::write(&file, drift)?;
        let output = command(directory.path(), "--check")?;
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr)?;
        assert!(error.contains(path));
        assert!(error.contains("first difference at byte"));
        let text = String::from_utf8(original.clone())?;
        let token = removal_token(path, &text)?;
        assert!(text.contains(&token));
        let removed = text.replacen(&token, "", 1);
        fs::write(&file, removed)?;
        let output = command(directory.path(), "--check")?;
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr)?;
        assert!(error.contains(path));
        assert!(error.contains("first difference at byte"));
        fs::write(file, original)?;
        Ok(())
    })?;
    Ok(())
}

fn command(root: &Path, mode: &str) -> Result<std::process::Output, Box<dyn Error>> {
    Ok(
        std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
            .args(["iam-policy", "--root"])
            .arg(root)
            .args((!mode.is_empty()).then_some(mode))
            .output()?,
    )
}

#[test]
fn given_reference_gap_when_pinned_then_default_action_is_granted_as_unconfirmed() -> TestResult {
    let ops = [Operation::new("s3", "ListBuckets", "s3")];
    let gap = "s3\tListBuckets\ts3\tunconfirmed: absent from the service authorization reference at 2026-11-24\n";
    let output = render(&ops, "", REFERENCE, gap)?;
    assert_eq!(
        output.unconfirmed,
        std::collections::BTreeSet::from(["s3/ListBuckets".to_owned()])
    );
    assert_eq!(
        output.actions,
        std::collections::BTreeSet::from(["s3:ListBuckets".to_owned()])
    );
    assert_eq!(output.policies.len(), 1);
    assert_eq!(
        output.gap_report,
        "The policy grants default-name actions for these 1 operations as unconfirmed; the pinned reference does not publish them.\ns3/ListBuckets\n"
    );
    assert!(render(&ops, MAPPINGS, REFERENCE, gap).is_err());
    let published = format!("{REFERENCE}s3\tListBuckets\tread\n");
    assert!(render(&ops, "", &published, gap).is_err());
    assert!(render(&[], "", REFERENCE, gap).is_err());
    assert!(
        render(
            &ops,
            "",
            REFERENCE,
            &gap.replace("2026-11-24", "2026-11-23")
        )
        .is_err()
    );
    assert!(
        render(
            &ops,
            "",
            REFERENCE,
            &gap.replace("\ts3\tunconfirmed", "\tec2\tunconfirmed")
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn given_single_action_alias_when_rendered_then_policy_envelopes_and_documents_match() -> TestResult
{
    let output = render(
        &[Operation::new("s3", "ListBuckets", "s3")],
        MAPPINGS,
        REFERENCE,
        "",
    )?;
    assert_eq!(
        output.actions,
        std::collections::BTreeSet::from(["s3:ListAllMyBuckets".to_owned()])
    );
    assert_documents(&output.text, &output.template)?;
    Ok(())
}

#[test]
fn given_unbounded_or_duplicate_reference_rows_when_rendered_then_rejected() {
    let ops = [Operation::new("s3", "ListBuckets", "s3")];
    [
        "# Retrieved: 2026-11-24\ns3\t*\tread\n",
        "# Retrieved: 2026-11-24\ns3\tListAllMyBuckets\tread\ns3\tListAllMyBuckets\tread\n",
        "# Retrieved: 2026-11-24\ns3\tListAllMyBuckets\tunknown\n",
    ]
    .iter()
    .for_each(|reference| assert!(render(&ops, MAPPINGS, reference, "").is_err()));
}

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","traits":{"aws.api#service":{"sdkId":"EC2","endpointPrefix":"ec2","arnNamespace":"ec2"},"aws.auth#sigv4":{"name":"ec2"},"aws.protocols#restJson1":{}}},
"n#DescribeVpcs":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/vpcs"}}},
"n#DeleteVpc":{"type":"operation"},
"n#DescribeSubnets":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/subnets"}}},
"n#DescribeSecrets":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/secrets"}}},
"n#DescribeDeferred":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/deferred"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#Item":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn callable_catalog(added: bool) -> Result<Vec<u8>, Box<dyn Error>> {
    use catalog_tool::{overlay, runtime_catalog, smithy};
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("models/ec2/service/2020-01-01");
    fs::create_dir_all(&source)?;
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    let mut operations = vec!["DescribeVpcs"];
    if added {
        operations.push("DescribeSubnets");
    } else {
        model
            .pointer_mut("/shapes")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("fixture shapes absent")?
            .remove("n#DescribeSubnets");
    }
    fs::write(source.join("model.json"), serde_json::to_vec(&model)?)?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.paginator = Some(overlay::Paginator::None);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    let mut policy_operations: Vec<_> = operations
        .iter()
        .map(|operation| (*operation, plan.clone()))
        .collect();
    let mut denied = overlay::OperationOverlay::default();
    denied.class = Some(overlay::OverlayClass::Deny(
        catalog_tool::classes::DenyReason::Secret,
    ));
    policy_operations.push(("DescribeSecrets", denied));
    let mut deferred = overlay::OperationOverlay::default();
    deferred.class = Some(overlay::OverlayClass::Defer);
    deferred.defer_reason = Some("fixture deferral".into());
    policy_operations.push(("DescribeDeferred", deferred));
    let policies = [overlay::ServiceOverlay::new("ec2", policy_operations)?];
    let checked = overlay::check_policies(
        directory.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse("Id\nItems\n")?;
    let base = smithy::build_catalog(directory.path(), &fields, "iam-policy-fixture")?;
    let kinds = fixture_kinds(&operations)?;
    let endpoints = std::collections::BTreeMap::from([("ec2".to_owned(), fixture_endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn fixture_kinds(operations: &[&str]) -> Result<catalog_tool::kind_map::KindMap, Box<dyn Error>> {
    use catalog_tool::{kind_map, names};
    let operation_kinds: Vec<_> = operations
        .iter()
        .map(|operation| kind_map::OperationMapping {
            service: "ec2",
            operation,
            kind: "AWS::EC2::VPC",
        })
        .collect();
    let kinds = kind_map::MappingSource {
        cfn: &[kind_map::CfnMapping::Mapped {
            cfn_type: "AWS::EC2::VPC",
            kind: "AWS::EC2::VPC",
            display_name_path: None,
        }],
        operations: &operation_kinds,
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?;
    Ok(kinds)
}

fn fixture_endpoint() -> Result<catalog_tool::endpoints::VerifiedEndpoint, Box<dyn Error>> {
    use catalog_tool::endpoints;
    use serde_json::json;
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"ec2","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://ec2.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("iam-policy-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://ec2.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("ec2"))
        .map_err(|_| "fixture endpoint evidence failed".into())
}

fn policy_actions(
    documents: &[serde_json::Value],
) -> Result<std::collections::BTreeSet<String>, Box<dyn Error>> {
    documents.iter().try_fold(
        std::collections::BTreeSet::new(),
        |mut actions, document| {
            assert_eq!(
                document.get("Version").and_then(serde_json::Value::as_str),
                Some("2012-10-17")
            );
            let statements = document
                .get("Statement")
                .and_then(serde_json::Value::as_array)
                .ok_or("statements absent")?;
            assert_eq!(statements.len(), 1);
            let statement = statements.first().ok_or("statement absent")?;
            assert_eq!(
                statement.get("Effect").and_then(serde_json::Value::as_str),
                Some("Allow")
            );
            assert_eq!(
                statement
                    .get("Resource")
                    .and_then(serde_json::Value::as_str),
                Some("*")
            );
            let names = statement
                .get("Action")
                .and_then(serde_json::Value::as_array)
                .ok_or("actions absent")?;
            assert!(!names.is_empty());
            names.iter().try_for_each(|name| -> TestResult {
                actions.insert(name.as_str().ok_or("non-string action")?.to_owned());
                Ok(())
            })?;
            Ok(actions)
        },
    )
}

fn assert_documents(bundle: &str, template: &str) -> TestResult {
    let bundle: Vec<serde_json::Value> = serde_json::from_str(bundle)?;
    let role = template
        .lines()
        .filter_map(|line| line.strip_prefix("      PolicyDocument: "))
        .map(serde_json::from_str)
        .collect::<Result<Vec<serde_json::Value>, _>>()?;
    assert!(!bundle.is_empty());
    assert_eq!(bundle, role);
    assert_eq!(policy_actions(&bundle)?, policy_actions(&role)?);
    Ok(())
}

#[test]
fn given_fresh_catalog_when_written_then_bundle_and_role_match_the_callable_catalog() -> TestResult
{
    let directory = fixture_root()?;
    let expected = fixture_policy()?;
    iam_policy::run(directory.path(), catalog_tool::ReportMode::Write)?;
    iam_policy::run(directory.path(), catalog_tool::ReportMode::Check)?;
    let bundle = fs::read_to_string(directory.path().join("catalog/iam-read-only-policy.json"))?;
    let role = fs::read_to_string(directory.path().join("deploy/read-only-role.yaml"))?;
    assert_eq!(bundle, expected.text);
    assert_eq!(role, expected.template);
    assert_documents(&bundle, &role)?;
    Ok(())
}

#[test]
fn given_multiple_policies_when_rendered_then_every_policy_is_attached_exactly_once() -> TestResult
{
    let names: Vec<_> = (0..400).map(|number| format!("List{number:04}")).collect();
    let ops: Vec<_> = names
        .iter()
        .map(|name| Operation::new("fixture", name, "fixture"))
        .collect();
    let reference = format!(
        "# Retrieved: 2026-11-24\n{}",
        names
            .iter()
            .map(|name| format!("fixture\t{name}\tread\n"))
            .collect::<String>()
    );
    let output = render(&ops, "", &reference, "")?;
    assert!(output.policies.len() >= 2);
    let resources: std::collections::BTreeSet<_> = output
        .template
        .lines()
        .filter_map(|line| {
            line.strip_prefix("  ")
                .and_then(|line| line.strip_suffix(':'))
        })
        .filter(|name| name.starts_with("InventoryReadOnlyPolicy"))
        .collect();
    let attachments: Vec<_> = output
        .template
        .lines()
        .filter_map(|line| line.strip_prefix("        - !Ref "))
        .collect();
    assert_eq!(attachments.len(), output.policies.len());
    assert_eq!(resources.len(), output.policies.len());
    assert_eq!(
        attachments
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        resources
    );
    assert_documents(&output.text, &output.template)?;
    Ok(())
}

#[test]
fn given_malformed_or_duplicate_exceptions_when_rendered_then_rejected() {
    let ops = [Operation::new("s3", "ListBuckets", "s3")];
    let row = "s3\tListBuckets\ts3\tunconfirmed: absent from the service authorization reference at 2026-11-24\n";
    [
        format!("{row}{row}"),
        "s3\tListBuckets\n".to_owned(),
        row.replacen("s3\t", "\t", 1),
        row.replace("ListBuckets", ""),
        row.replace("\ts3\t", "\t\t"),
    ]
    .iter()
    .for_each(|text| assert!(render(&ops, "", REFERENCE, text).is_err()));
    assert!(render(&[], "", REFERENCE, "").is_err());
}

#[test]
fn given_read_named_fixture_deny_decision_when_generated_then_denied_action_is_not_granted()
-> TestResult {
    let bytes = callable_catalog(false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(catalog.lookup("ec2", "DescribeSecrets").is_none());
    assert!(
        catalog
            .decisions()
            .iter()
            .any(|decision| decision.service == "ec2"
                && decision.operation == "DescribeSecrets"
                && matches!(
                    decision.disposition,
                    inventory_core::catalog::runtime::ReviewDisposition::Denied { .. }
                ))
    );
    let output = render(&iam_policy::operations(&catalog)?, "", REFERENCE, "")?;
    assert!(!output.actions.contains("ec2:DescribeSecrets"));
    assert!(!output.operation_actions.contains_key("ec2/DescribeSecrets"));
    Ok(())
}

fn removal_token(path: &str, text: &str) -> Result<String, Box<dyn Error>> {
    if path.ends_with("policy-gaps.txt") {
        return Ok(text
            .lines()
            .nth(1)
            .ok_or("unconfirmed entry absent")?
            .to_owned());
    }
    let documents: Vec<serde_json::Value> = if path.ends_with(".json") {
        serde_json::from_str(text)?
    } else {
        text.lines()
            .filter_map(|line| line.strip_prefix("      PolicyDocument: "))
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?
    };
    let actions = policy_actions(&documents)?;
    let middle = actions.len().div_euclid(2);
    actions
        .into_iter()
        .nth(middle)
        .ok_or_else(|| "action token absent".into())
}

#[test]
fn given_print_mode_when_invoked_then_stdout_is_only_the_json_bundle() -> TestResult {
    let directory = fixture_root()?;
    let output = command(directory.path(), "")?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let printed: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)?;
    let bundle: Vec<serde_json::Value> = serde_json::from_str(&fixture_policy()?.text)?;
    assert_eq!(printed, bundle);
    Ok(())
}

#[test]
fn given_write_named_unconfirmed_row_when_rendered_then_generation_fails_naming_the_row()
-> TestResult {
    let ops = [Operation::new("ec2", "DeleteFoo", "ec2")];
    let row = "ec2\tDeleteFoo\tec2\tunconfirmed: absent from the service authorization reference at 2026-11-24\n";
    let error = render(&ops, "", REFERENCE, row)
        .err()
        .ok_or("unconfirmed write action was granted")?;
    let message = error.to_string();
    assert!(message.contains("ec2/DeleteFoo"));
    assert!(message.contains("not read-only"));
    Ok(())
}

#[test]
fn given_real_unconfirmed_rows_when_rendered_then_each_read_named_default_is_granted() -> TestResult
{
    let reference = fs::read_to_string(root().join(iam_policy::REFERENCE_PATH))?;
    let text = fs::read_to_string(root().join(iam_policy::EXCEPTIONS_PATH))?;
    let operations = text
        .lines()
        .filter(|row| !row.starts_with('#'))
        .map(|row| -> Result<Operation<'_>, Box<dyn Error>> {
            let mut fields = row.split('\t');
            let (Some(service), Some(name), Some(prefix), Some(_), None) = (
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
            ) else {
                return Err("invalid real unconfirmed row".into());
            };
            Ok(Operation::new(service, name, prefix))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let output = render(&operations, "", &reference, &text)?;
    assert_eq!(operations.len(), 14);
    for operation in operations {
        let action = format!("{}:{}", operation.prefix, operation.name);
        assert_eq!(
            output
                .operation_actions
                .get(&format!("{}/{}", operation.service, operation.name)),
            Some(&action)
        );
        assert!(output.actions.contains(&action));
    }
    Ok(())
}

fn fixture_root() -> Result<tempfile::TempDir, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    fs::create_dir_all(directory.path().join("catalog/overlay"))?;
    fs::create_dir_all(directory.path().join("deploy"))?;
    fs::write(
        directory.path().join("catalog/aws-api.catalog"),
        callable_catalog(false)?,
    )?;
    [
        (iam_policy::MAPPINGS_PATH, ""),
        (iam_policy::REFERENCE_PATH, CLI_REFERENCE),
        (iam_policy::EXCEPTIONS_PATH, CLI_GAPS),
    ]
    .iter()
    .try_for_each(|(path, text)| fs::write(directory.path().join(path), text))?;
    Ok(directory)
}

fn fixture_policy() -> Result<iam_policy::Rendered, iam_policy::Error> {
    render(
        &[Operation::new("ec2", "DescribeVpcs", "ec2")],
        "",
        CLI_REFERENCE,
        CLI_GAPS,
    )
}

#[test]
fn given_shared_actions_when_written_then_denied_findings_precede_deferred_rows()
-> Result<(), Box<dyn Error>> {
    let root = fixture_root()?;
    fs::write(
        root.path().join(iam_policy::MAPPINGS_PATH),
        "[operation]\n\"ec2/DescribeSecrets\" = \"ec2:DescribeVpcs\"\n\
         \"ec2/DescribeDeferred\" = \"ec2:DescribeVpcs\"\n",
    )?;
    iam_policy::run(root.path(), catalog_tool::ReportMode::Write)?;
    let report = fs::read_to_string(root.path().join("deploy/read-only-policy-gaps.txt"))?;
    assert!(
        report.ends_with(
            "\nshared\nActions use explicit operation overrides or model signing-name:operation; \
             non-callable operations whose IAM action is not derivable by that lookup are not listed.\n\
             count: 2 non-callable operations\n\
             DENIED (must-deny): 1\n\
             ec2/DescribeSecrets ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n\
             deferred: 1\n\
             ec2/DescribeDeferred ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n\
             unclassified: 0\n\
             \nbroad\nExisting aliases or callable mappings associate these granted actions with \
             multiple operation IDs, so they can permit other requests.\ncount: 1 actions\n\
             ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n"
        ),
        "{report}"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("catalog/iam-read-only-policy.json"))?,
        fixture_policy()?.text
    );
    iam_policy::run(root.path(), catalog_tool::ReportMode::Check)?;
    Ok(())
}

#[test]
fn given_two_callable_owners_when_shared_then_rows_are_sorted_and_operations_counted_once()
-> TestResult {
    let root = fixture_root()?;
    fs::write(
        root.path().join("catalog/aws-api.catalog"),
        callable_catalog(true)?,
    )?;
    fs::write(root.path().join(iam_policy::REFERENCE_PATH), REFERENCE)?;
    fs::write(root.path().join(iam_policy::EXCEPTIONS_PATH), "")?;
    fs::write(
        root.path().join(iam_policy::MAPPINGS_PATH),
        "[operation]\n\"ec2/DescribeSecrets\" = \"ec2:DescribeVpcs\"\n\
         \"ec2/DescribeSubnets\" = \"ec2:DescribeVpcs\"\n\
         \"ec2/DescribeDeferred\" = \"ec2:DescribeVpcs\"\n",
    )?;
    iam_policy::run(root.path(), catalog_tool::ReportMode::Write)?;
    let report = fs::read_to_string(root.path().join("deploy/read-only-policy-gaps.txt"))?;
    assert!(
        report.contains("\nbroad\n"),
        "shared alias grants must identify their broader request surface: {report}"
    );
    assert!(
        report.ends_with(
            "\nshared\nActions use explicit operation overrides or model signing-name:operation; \
             non-callable operations whose IAM action is not derivable by that lookup are not listed.\n\
             count: 2 non-callable operations\n\
             DENIED (must-deny): 1\n\
             ec2/DescribeSecrets ec2:DescribeVpcs callable-by ec2/DescribeSubnets\n\
             ec2/DescribeSecrets ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n\
             deferred: 1\n\
             ec2/DescribeDeferred ec2:DescribeVpcs callable-by ec2/DescribeSubnets\n\
             ec2/DescribeDeferred ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n\
             unclassified: 0\n\
             \nbroad\nExisting aliases or callable mappings associate these granted actions with \
             multiple operation IDs, so they can permit other requests.\ncount: 1 actions\n\
             ec2:DescribeVpcs callable-by ec2/DescribeSubnets\n\
             ec2:DescribeVpcs callable-by ec2/DescribeVpcs\n"
        ),
        "{report}"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("catalog/iam-read-only-policy.json"))?,
        fixture_policy()?.text
    );
    Ok(())
}
