#[path = "../src/smithy/compile/tests/git.rs"]
mod git;
#[path = "support/skip_runtime.rs"]
mod skip_runtime;
use catalog_tool::overlay::{Origin, ServiceOverlay};
use catalog_tool::runtime_catalog::RuntimeCatalogCompileError;
use inventory_core::catalog::runtime::RuntimeCatalog;
use skip_runtime::Case;
use std::{fs, process::Command};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ENTRY: &str = "[operation.ListThings]\nclass = \"enumerate\"\nskip = [\"DescribeThing\"]\n";
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Svc":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Example","endpointPrefix":"example"},"aws.protocols#awsJson1_1":{}}},
"n#ListThings":{"type":"operation"}
}}"#;

#[test]
fn given_hand_written_skip_without_evidence_when_parsed_then_evidence_error_is_returned()
-> TestResult {
    for (origin, reviewer) in [(Origin::Hand, "reviewer"), (Origin::Generated, "generated")] {
        let text =
            format!("format = 1\nreviewed = \"2026-11-24\"\nreviewer = \"{reviewer}\"\n{ENTRY}");
        let error = ServiceOverlay::parse("example", &text, origin)
            .err()
            .ok_or("skip without evidence accepted")?;
        assert!(
            error.to_string().contains(
                "skip_evidence: each skip requires compared, positive resources, and when"
            ),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn given_hand_written_skip_without_evidence_when_cli_checks_or_compiles_then_evidence_error_is_returned()
-> TestResult {
    let source = git::repository(MODEL)?;
    let root = tempfile::tempdir()?;
    let directory = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("example.toml"),
        format!("format = 1\nreviewed = \"2026-11-24\"\nreviewer = \"generated\"\n{ENTRY}"),
    )?;
    fs::write(root.path().join("catalog/field-ids.txt"), "Name\n")?;
    fs::write(
        root.path().join("catalog/cloudcontrol-universal.json"),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"),
    )?;
    fs::write(root.path().join(catalog_tool::relations::ROWS_PATH), b"")?;
    for arguments in [
        vec!["overlay", "--check"],
        vec!["models", "compile", "--check"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
            .args(arguments)
            .arg("--root")
            .arg(root.path())
            .arg("--source")
            .arg(source.path())
            .output()?;
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr)?;
        assert!(
            error.contains(
                "skip_evidence: each skip requires compared, positive resources, and when"
            ),
            "{error}"
        );
    }
    Ok(())
}

const KIND_OVERLAY: &str = r#"format = 1
reviewed = "2026-11-24"
reviewer = "generated"
[operation.ListThings]
class = "enumerate"
scope = "region"
items = "Items"
skip = ["DescribeThing"]
identity = { key = ["Id"] }
paginator = { kind = "none" }
[operation.ListThings.skip_evidence.DescribeThing]
service = "example"
list = "ListThings"
describe = "DescribeThing"
inputs_sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
compared = "seeded modeled members and raw values"
resources = 1
when = "2026-11-24"
[operation.DescribeThing]
class = "expand"
scope = "region"
items = "Items"
identity = { key = ["Id"] }
paginator = { kind = "none" }
bind = { Id = { from = "ListThings", path = "Id" } }
"#;

#[test]
fn given_different_kind_skip_when_cli_checks_and_compiles_then_kind_loss_is_refused() -> TestResult
{
    let model = serde_json::json!({"smithy":"2.0","shapes":{
        "n#Svc":{"type":"service","version":"1","operations":[{"target":"n#ListThings"},{"target":"n#DescribeThing"}],"traits":{"aws.api#service":{"sdkId":"Example","endpointPrefix":"example"},"aws.protocols#awsJson1_1":{}}},
        "n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
        "n#DescribeThing":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
        "n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
        "n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
        "n#Items":{"type":"list","member":{"target":"n#Item"}},
        "n#Item":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
    }});
    let source = git::repository(&serde_json::to_string(&model)?)?;
    let root = tempfile::tempdir()?;
    let directory = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("example.toml"), KIND_OVERLAY)?;
    fs::write(
        root.path().join("catalog/kinds.generated.tsv"),
        "api\tAWS::ApiOnly::SkipExampleParent\tkind\tSkip kind contract\napi\tAWS::ApiOnly::SkipExampleChild\tkind\tSkip kind contract\noperation\texample\tListThings\tAWS::ApiOnly::SkipExampleParent\noperation\texample\tDescribeThing\tAWS::ApiOnly::SkipExampleChild\n",
    )?;
    fs::write(root.path().join("catalog/field-ids.txt"), "Id\nItems\n")?;
    fs::write(
        root.path().join("catalog/cloudcontrol-universal.json"),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"),
    )?;
    fs::write(root.path().join(catalog_tool::relations::ROWS_PATH), b"")?;
    for arguments in [
        vec!["overlay", "--check"],
        vec!["models", "compile", "--check"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
            .args(arguments)
            .arg("--root")
            .arg(root.path())
            .arg("--source")
            .arg(source.path())
            .output()?;
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr)?;
        assert!(
            error.contains("skip parent does not emit child kind"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn given_receipt_for_another_pair_when_compiled_then_no_skip_is_authorized() -> TestResult {
    let bytes = skip_runtime::catalog(Case::Ordinary)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let mut evidence = skip_runtime::receipt(&catalog)?;
    assert!(skip_runtime::catalog_with(Case::Ordinary, Some(&evidence), None, |_| {}).is_ok());
    evidence.list = catalog_tool::overlay::OperationName::new("DescribeRegions")?;
    assert!(skip_runtime::catalog_with(Case::Ordinary, Some(&evidence), None, |_| {}).is_err());
    Ok(())
}

#[test]
fn given_catalog_inputs_changed_when_compiled_then_the_old_receipt_cannot_authorize_skip()
-> TestResult {
    let old = skip_runtime::catalog(Case::Ordinary)?;
    let old_catalog = RuntimeCatalog::from_bytes(&old)?;
    let stale = skip_runtime::receipt(&old_catalog)?;
    assert!(
        skip_runtime::catalog_with(Case::AdditionalEmission, Some(&stale), None, |_| {}).is_err()
    );
    let current = skip_runtime::catalog(Case::AdditionalEmission)?;
    let catalog = RuntimeCatalog::from_bytes(&current)?;
    let parent = catalog
        .lookup("skipruntime", "ListThings")
        .ok_or("parent missing")?;
    assert_eq!(parent.skips(), &[Box::<str>::from("DescribeThing")]);
    Ok(())
}

#[test]
fn given_child_only_tag_source_when_compiled_then_skip_is_refused() -> TestResult {
    let tags = catalog_tool::overlay::MemberPath::new("Tags")?;
    let child_tags = |plan: &mut catalog_tool::overlay::OperationOverlay| {
        plan.tags = Some(tags.clone());
    };
    let bytes = skip_runtime::catalog_with(Case::Ordinary, None, None, child_tags)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let parent = catalog
        .lookup("skipruntime", "ListThings")
        .ok_or("parent missing")?;
    let child = catalog
        .lookup("skipruntime", "DescribeThing")
        .ok_or("child missing")?;
    assert!(parent.resource_emissions().primary().tags().is_none());
    assert!(child.resource_emissions().primary().tags().is_some());
    let receipt = skip_runtime::receipt(&catalog)?;
    let error = skip_runtime::catalog_with(Case::Skipped, Some(&receipt), None, child_tags)
        .err()
        .ok_or("child tag source silently removed")?;
    assert!(matches!(
        error.downcast_ref::<RuntimeCatalogCompileError>(),
        Some(RuntimeCatalogCompileError::CapabilityContext { path, .. })
            if path.contains("tag source") && path.contains("DescribeThing")
    ));
    Ok(())
}

#[test]
fn given_discovery_only_child_branch_when_compiled_then_skip_is_refused() -> TestResult {
    use catalog_tool::overlay::{Emission, ExecutionBranch, OperationOverlay, Scope};
    let child_branches = |plan: &mut OperationOverlay| {
        plan.branches = [
            ("metadata", Emission::EmitMetadata),
            ("discovery", Emission::DiscoveryOnly),
        ]
        .into_iter()
        .map(|(name, emission)| ExecutionBranch {
            discriminator: name.into(),
            fixed: Default::default(),
            bind: Default::default(),
            emission: Some(emission),
            row_policy: None,
            scope: Scope::Region,
            request_region: None,
        })
        .collect();
    };
    let bytes = skip_runtime::catalog_with(Case::Ordinary, None, None, child_branches)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let child = catalog
        .lookup("skipruntime", "DescribeThing")
        .ok_or("child missing")?;
    assert!(child.branches().iter().any(|branch| {
        child.effective_emission(Some(branch))
            == inventory_core::catalog::runtime::Emission::DiscoveryOnly
    }));
    let receipt = skip_runtime::receipt(&catalog)?;
    let error = skip_runtime::catalog_with(Case::Skipped, Some(&receipt), None, child_branches)
        .err()
        .ok_or("skip accepted a discovery-only child branch")?;
    assert!(matches!(
        error.downcast_ref::<RuntimeCatalogCompileError>(),
        Some(RuntimeCatalogCompileError::CapabilityContext { path, .. })
            if path == "skip DescribeThing branch resource scope or emission differs"
    ));
    Ok(())
}

#[test]
fn given_cross_service_consumer_when_compiled_then_local_skip_keeps_its_identity() -> TestResult {
    let bytes = skip_runtime::catalog(Case::CrossService)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let parent = catalog
        .lookup("skipruntime", "ListThings")
        .ok_or("parent missing")?;
    assert_eq!(parent.skips(), &[Box::<str>::from("DescribeThing")]);
    assert!(catalog.lookup("otherskip", "DescribeThing").is_some());
    Ok(())
}

#[test]
fn given_bootstrap_carrier_when_generating_catalog_then_skip_is_refused_before_encoding()
-> TestResult {
    use catalog_tool::kind_map::BootstrapMapping;
    let bytes = skip_runtime::catalog(Case::Ordinary)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let receipt = skip_runtime::receipt(&catalog)?;
    for bootstrap in [
        BootstrapMapping::AccountIdentity {
            service: "skipruntime",
            operation: "ListThings",
            account_path: "Account",
        },
        BootstrapMapping::RegionDiscovery {
            service: "skipruntime",
            operation: "ListThings",
            regions_path: "Items",
            region_name_path: "Id",
            opt_in_status_path: None,
        },
    ] {
        let result =
            skip_runtime::catalog_with(Case::Skipped, Some(&receipt), Some(bootstrap), |_| {});
        let error = result.err().ok_or("bootstrap carrier was authorized")?;
        assert!(matches!(
            error.downcast_ref::<RuntimeCatalogCompileError>(),
            Some(RuntimeCatalogCompileError::CapabilityContext { operation, .. })
                if operation == "skip"
        ));
    }
    Ok(())
}
