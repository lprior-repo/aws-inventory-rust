use catalog_tool::{
    classes::DenyList,
    overlay::{self, CheckError, Identity, ItemSelector, OperationOverlay, OverlayClass, Paginator, ServiceOverlay, ViolationRule},
};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn fixture() -> Result<TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let models = root.path().join("source/models/things/service/2020-01-01");
    fs::create_dir_all(&models)?;
    fs::create_dir_all(root.path().join("catalog"))?;
    fs::write(models.join("things.json"), MODEL)?;
    Ok(root)
}

fn source(root: &Path) -> std::path::PathBuf { root.join("source") }

fn policy(items: &str, identity: &str) -> Result<ServiceOverlay, overlay::NameError> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.items = Some(ItemSelector::parse(items)?);
    plan.paginator = Some(Paginator::None);
    plan.identity = Some(Identity::Key(vec![overlay::MemberPath::selected(identity)?]));
    ServiceOverlay::new("things", vec![("ListThings", plan)])
}

fn check(root: &Path, policies: &[ServiceOverlay]) -> Result<overlay::CheckedOverlays, CheckError> {
    overlay::check_policies(&source(root), policies, &DenyList::builtin())
}

#[test]
fn item_selector_accepts_explicit_root_without_changing_member_paths() {
    assert_eq!(ItemSelector::parse("Envelope.Items"),
        overlay::MemberPath::new("Envelope.Items").map(ItemSelector::Member));
    assert_eq!(ItemSelector::parse("$"), Ok(ItemSelector::Root));
    assert!(ItemSelector::parse("$.").is_err());
    assert_eq!(overlay::MemberPath::selected("$").map(|path| path.to_string()), Ok("$".to_owned()));
    assert!(overlay::MemberPath::new("$").is_err());
}

#[test]
fn explicit_empty_policy_builds_a_non_callable_baseline() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let checked = check(root.path(), &[])?;
    assert!(checked.services().is_empty());
    assert_eq!(checked.report(),
        "totals\tservices_with_overlay=0\tcallable_enumerate=0\tcallable_expand=0\tdenied_by_overlay=0\tdeferred=0\tservices_without_overlay=1\tunreviewed_enumerate=1\tunreviewed_expand=0\tunreviewed_unclassified=0\n");
    Ok(())
}

#[test]
fn embedded_cli_publishes_source_subset_without_authorizing_unreviewed_calls() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let report = root.path().join("catalog/aws-overlay.report");
    fs::write(&report, b"prior\n")?;
    for mode in ["--write", "--check"] {
        let output = Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
            .args(["overlay", "--root"]).arg(root.path())
            .arg("--source").arg(source(root.path())).arg(mode).output()?;
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, b"");
        let text = fs::read_to_string(&report)?;
        assert!(text.contains("services_with_overlay=0"));
        assert!(text.contains("callable_enumerate=0\tcallable_expand=0"));
        assert!(text.contains("services_without_overlay=1\tunreviewed_enumerate=1"));
    }
    Ok(())
}

#[test]
fn reviewed_list_exposes_exact_readonly_callable_plan() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let checked = check(root.path(), &[policy("Items", "Id")?])?;
    let service = checked.services().first().ok_or("missing reviewed service")?;
    let callable = service.callable().first().ok_or("missing callable")?;
    assert_eq!(callable.operation(), "ListThings");
    assert_eq!(callable.class(), overlay::EffectiveClass::Enumerate);
    assert_eq!(callable.plan().items, Some(ItemSelector::Member(overlay::MemberPath::new("Items")?)));
    assert_eq!(callable.plan().paginator, Some(Paginator::None));
    assert_eq!(callable.plan().identity, Some(Identity::Key(vec![overlay::MemberPath::new("Id")?])));
    assert_eq!(checked.report(),
        "totals\tservices_with_overlay=1\tcallable_enumerate=1\tcallable_expand=0\tdenied_by_overlay=0\tdeferred=0\tservices_without_overlay=0\tunreviewed_enumerate=0\tunreviewed_expand=0\tunreviewed_unclassified=0\noperation\tthings\tListThings\tenumerate\tclass=overlay,items=overlay,page_size=missing,paginator=overlay,identity=overlay,bindings=missing,scope=missing,batch=missing\n");
    Ok(())
}

#[test]
fn duplicate_service_policy_is_rejected_before_a_conflicting_denial_can_be_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let mut deny = OperationOverlay::default();
    deny.class = Some(OverlayClass::Deny(catalog_tool::classes::DenyReason::Content));
    let denial = ServiceOverlay::new("things", vec![("ListThings", deny)])?;
    for policies in [vec![policy("Items", "Id")?, denial.clone()], vec![denial, policy("Items", "Id")?]] {
        assert!(matches!(check(root.path(), &policies),
            Err(CheckError::Invalid { service, key, rule, .. })
                if service == "things" && key == "service" && rule == "duplicate service policy"));
    }
    Ok(())
}

#[test]
fn custom_policy_still_rejects_unknown_service_names() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let policies = [ServiceOverlay::new("missing", Vec::new())?];
    match check(root.path(), &policies) {
        Err(CheckError::Violations(violations)) => assert!(violations.iter().any(|violation|
            violation.rule() == ViolationRule::UnknownService)),
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("unknown custom policy service accepted".into()),
    }
    Ok(())
}

#[test]
fn empty_injected_denials_cannot_disable_mandatory_secret_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    fs::write(source(root.path()).join("models/things/service/2020-01-01/things.json"),
        MODEL.replace("ListThings", "GetSecretValue"))?;
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.items = Some(ItemSelector::Root);
    plan.paginator = Some(Paginator::None);
    plan.identity = Some(Identity::Key(vec![overlay::MemberPath::new("Id")?]));
    let policies = [ServiceOverlay::new("things", vec![("GetSecretValue", plan)])?];
    match overlay::check_policies(&source(root.path()), &policies, &DenyList::default()) {
        Err(CheckError::Violations(violations)) => assert!(violations.iter().any(|violation|
            violation.operation() == "GetSecretValue" && violation.rule() == ViolationRule::OverlayPromotesDenied)),
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("mandatory secret operation became callable".into()),
    }
    Ok(())
}

#[test]
fn absent_and_empty_model_directories_are_not_an_empty_success() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    assert!(overlay::check_policies(root.path(), &[], &DenyList::builtin()).is_err());
    fs::create_dir(root.path().join("models"))?;
    assert!(matches!(overlay::check_policies(root.path(), &[], &DenyList::builtin()),
        Err(CheckError::ModelSource(catalog_tool::model_source::Error::NoModels))));
    Ok(())
}

#[test]
fn root_selector_uses_output_structure_not_item_structure() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let model = MODEL.replace("\"Id\":{\"target\":\"smithy.api#String\"},\"Items\"",
        "\"RootId\":{\"target\":\"smithy.api#String\"},\"Items\"")
        .replace("\"n#Thing\":{\"type\":\"structure\",\"members\":{\"Id\"",
            "\"n#Thing\":{\"type\":\"structure\",\"members\":{\"ItemId\"");
    fs::write(source(root.path()).join("models/things/service/2020-01-01/things.json"), model)?;
    let checked = check(root.path(), &[policy("$", "RootId")?])?;
    let callable = checked.services().first().ok_or("service")?.callable().first().ok_or("callable")?;
    assert_eq!(callable.plan().items, Some(ItemSelector::Root));
    assert_eq!(callable.plan().identity, Some(Identity::Key(vec![overlay::MemberPath::new("RootId")?])));
    let errors = match check(root.path(), &[policy("$", "ItemId")?]) {
        Err(CheckError::Violations(errors)) => errors,
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("root selector accepted item-only identity".into()),
    };
    assert_eq!(errors.iter().map(|error| (error.key(), error.value(), error.rule())).collect::<Vec<_>>(),
        [("identity", "ItemId", ViolationRule::InvalidPath)]);
    Ok(())
}
