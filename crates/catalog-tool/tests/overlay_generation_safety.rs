use catalog_tool::{
    classes::{self, Class, DenyList, OperationFacts},
    overlay::{
        self, CheckError, Origin, OverlayClass, ReviewDisposition, ServiceOverlay, ViolationRule,
        generate::{Generated, Generator},
    },
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::LazyLock};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
static MODELS: LazyLock<std::result::Result<Generator, String>> =
    LazyLock::new(|| Generator::read(&source()).map_err(|error| error.to_string()));
static GENERATED: LazyLock<std::result::Result<Generated, String>> =
    LazyLock::new(|| generate_all().map_err(|error| error.to_string()));

fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models")
}

fn generator() -> Result<&'static Generator> {
    match &*MODELS {
        Ok(generator) => Ok(generator),
        Err(reason) => Err(reason.as_str().into()),
    }
}

fn generated() -> Result<&'static Generated> {
    match &*GENERATED {
        Ok(generated) => Ok(generated),
        Err(reason) => Err(reason.as_str().into()),
    }
}

fn generate_all() -> Result<Generated> {
    let generator = generator()?;
    let services = generator.values().keys().cloned().collect();
    generator.generate(&services, &DenyList::builtin())
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_all_pinned_models_when_generated_then_no_denied_operation_is_callable() -> Result {
    let generator = generator()?;
    let generated = generated()?;
    assert_eq!(generator.values().len(), 436);
    assert_eq!(generated.files().len(), 436);
    let deny = DenyList::builtin();
    for (service, value) in generator.values() {
        assert_denied_absent(service, value, generated, &deny)?;
    }
    Ok(())
}

fn assert_denied_absent(
    service: &str,
    value: &Value,
    generated: &Generated,
    deny: &DenyList<'_>,
) -> Result {
    let shapes = value
        .get("shapes")
        .and_then(Value::as_object)
        .ok_or("missing shapes")?;
    let policy = generated
        .policies()
        .iter()
        .find(|policy| policy.service() == service)
        .ok_or("missing generated service")?;
    for (id, shape) in shapes {
        if shape.get("type").and_then(Value::as_str) != Some("operation") {
            continue;
        }
        assert_denied_operation(service, id, policy, deny);
    }
    Ok(())
}

fn assert_denied_operation(service: &str, id: &str, policy: &ServiceOverlay, deny: &DenyList<'_>) {
    let name = id.rsplit_once('#').map_or(id, |(_, name)| name);
    let facts = OperationFacts {
        service,
        name,
        has_required_input: false,
        readonly: false,
    };
    if matches!(classes::classify(&facts, deny), Class::Denied(_)) {
        assert!(!policy.operations().contains_key(name), "{service}/{name}");
    }
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_generated_file_when_denied_operation_is_promoted_then_checker_rejects_it() -> Result {
    let mut text = generated()?
        .files()
        .get("ec2")
        .ok_or("missing ec2 file")?
        .clone();
    text.push_str("\n[operation.CreateVpc]\nclass = \"enumerate\"\n");
    let policy = ServiceOverlay::parse("ec2", &text, Origin::Generated)?;
    let error = match generator()?.check(&[policy], &DenyList::builtin()) {
        Err(error) => error,
        Ok(_) => return Err("tampered policy was accepted".into()),
    };
    match error.downcast_ref::<CheckError>() {
        Some(CheckError::Violations(violations)) => assert!(violations.iter().any(|violation| {
            violation.operation() == "CreateVpc"
                && violation.rule() == ViolationRule::OverlayPromotesDenied
        })),
        other => return Err(format!("unexpected checker failure: {other:?}").into()),
    }
    Ok(())
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_new_deny_when_regenerated_then_previous_callable_disappears_and_check_passes() -> Result {
    let (service, name) = first_callable()?;
    let text = format!("{service}/{name}\tsecret\n");
    let deny = DenyList::parse(&text)?;
    let selection = BTreeSet::from([service.to_owned()]);
    let next = generator()?.generate(&selection, &deny)?;
    let policy = next
        .policies()
        .first()
        .ok_or("missing regenerated service")?;
    assert!(!policy.operations().contains_key(name));
    let checked = generator()?.check(next.policies(), &deny)?;
    assert_not_callable(&checked, name)?;
    Ok(())
}

fn assert_not_callable(checked: &overlay::CheckedOverlays, name: &str) -> Result {
    let reviewed = checked
        .services()
        .first()
        .ok_or("missing checked service")?;
    assert!(
        !reviewed
            .callable()
            .iter()
            .any(|operation| operation.operation() == name)
    );
    Ok(())
}

fn first_callable() -> Result<(&'static str, &'static str)> {
    for policy in generated()?.policies() {
        for (name, entry) in policy.operations() {
            if entry.class == Some(OverlayClass::Enumerate) {
                return Ok((policy.service(), name.as_str()));
            }
        }
    }
    Err("no generated callable candidate".into())
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_all_generated_services_when_merged_then_embedded_callable_plans_are_unchanged() -> Result {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    for (service, text) in generated()?.files() {
        fs::write(directory.join(format!("{service}.toml")), text)?;
    }
    let baseline = generator()?.check(&overlay::embedded()?, &DenyList::builtin())?;
    let merged = overlay::load(root.path())?;
    let checked = generator()?.check(&merged, &DenyList::builtin())?;
    assert_embedded_unchanged(&baseline, &checked)?;
    assert_thin_slice(&checked)
}

fn assert_embedded_unchanged(
    before: &overlay::CheckedOverlays,
    after: &overlay::CheckedOverlays,
) -> Result {
    for service in before.services() {
        let merged = after
            .services()
            .iter()
            .find(|entry| entry.service() == service.service())
            .ok_or("missing merged service")?;
        for operation in service.callable() {
            let next = merged
                .callable()
                .iter()
                .find(|next| next.operation() == operation.operation())
                .ok_or("missing original callable")?;
            assert_eq!(next, operation);
        }
    }
    Ok(())
}

fn assert_thin_slice(checked: &overlay::CheckedOverlays) -> Result {
    for (service, name) in [
        ("ec2", "DescribeVpcs"),
        ("ec2", "DescribeSubnets"),
        ("ec2", "DescribeRegions"),
        ("sts", "GetCallerIdentity"),
    ] {
        let review = checked
            .services()
            .iter()
            .find(|entry| entry.service() == service)
            .ok_or("missing thin slice service")?;
        assert!(
            review
                .callable()
                .iter()
                .any(|entry| entry.operation() == name)
        );
    }
    assert_sts_denied(checked)
}

fn assert_sts_denied(checked: &overlay::CheckedOverlays) -> Result {
    let sts = checked
        .services()
        .iter()
        .find(|entry| entry.service() == "sts")
        .ok_or("missing sts")?;
    for name in [
        "GetDelegatedAccessToken",
        "GetFederationToken",
        "GetWebIdentityToken",
    ] {
        let decision = sts
            .decisions()
            .iter()
            .find(|entry| entry.operation() == name)
            .ok_or("missing STS decision")?;
        assert_secret(decision.disposition());
    }
    Ok(())
}

fn assert_secret(disposition: &ReviewDisposition) {
    assert_eq!(
        disposition,
        &ReviewDisposition::Denied {
            reason: "secret".to_owned()
        }
    );
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_all_pinned_models_when_generated_again_then_every_file_is_byte_identical() -> Result {
    assert_eq!(generated()?.files(), generate_all()?.files());
    Ok(())
}

#[test]
#[ignore = "reads all 436 pinned models; run the documented overlay-safety gate"]
fn given_reviewed_denials_and_defers_when_generated_then_safety_decisions_remain_authoritative()
-> Result {
    let proposals = generated()?;
    for reviewed in overlay::embedded()? {
        let policy = proposals
            .policies()
            .iter()
            .find(|policy| policy.service() == reviewed.service())
            .ok_or("missing generated service")?;
        assert_reviewed_safety(&reviewed, policy);
    }
    Ok(())
}

fn assert_reviewed_safety(reviewed: &ServiceOverlay, generated: &ServiceOverlay) {
    for (name, entry) in reviewed.operations() {
        let proposal = generated.operations().get(name);
        match entry.class {
            Some(OverlayClass::Deny(_)) => {
                assert!(proposal.is_none(), "{}/{}", reviewed.service(), name);
            }
            Some(OverlayClass::Defer) if entry.generation_blocker.is_none() => {
                assert!(
                    proposal.is_none_or(|entry| entry.class == Some(OverlayClass::Defer)),
                    "{}/{}",
                    reviewed.service(),
                    name
                );
            }
            _ => {}
        }
    }
}
