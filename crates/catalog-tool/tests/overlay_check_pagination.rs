use catalog_tool::classes::DenyList;
use catalog_tool::overlay::{
    self, CheckError, Identity, ItemSelector, MemberPath, OperationOverlay, OverlayClass,
    Paginator, ServiceOverlay, ViolationRule,
};
use std::path::Path;

const PAGINATED_MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListPaginated"}]},
"n#ListPaginated":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},
    "traits":{"smithy.api#paginated":{"inputToken":"Next","outputToken":"Next"}}},
"n#Input":{"type":"structure","members":{"Next":{"target":"smithy.api#String"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"},"Next":{"target":"smithy.api#String"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

/// Operation with NeedsOverlay pagination (heuristic detection, no explicit trait).
/// Has input cursor field and output cursor field, so it's detected as paginated
/// but without the formal smithy.api#paginated trait.
const NEEDS_OVERLAY_MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListNeedsOverlay"}]},
"n#ListNeedsOverlay":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"NextToken":{"target":"smithy.api#String"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"},"NextToken":{"target":"smithy.api#String"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn paginated_fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let models = root
        .path()
        .join("source/models/paginated/service/2020-01-01");
    std::fs::create_dir_all(&models)?;
    std::fs::write(models.join("paginated.json"), PAGINATED_MODEL)?;
    Ok(root)
}

fn paginated_source(root: &Path) -> std::path::PathBuf {
    root.join("source")
}

fn paginated_policy() -> Result<ServiceOverlay, Box<dyn std::error::Error>> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.items = Some(ItemSelector::parse("Items")?);
    plan.paginator = Some(Paginator::None);
    plan.identity = Some(Identity::Key(vec![MemberPath::selected("Id")?]));
    Ok(ServiceOverlay::new(
        "paginated",
        vec![("ListPaginated", plan)],
    )?)
}

fn needs_overlay_fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let models = root.path().join("source/models/needs/service/2020-01-01");
    std::fs::create_dir_all(&models)?;
    std::fs::write(models.join("needs.json"), NEEDS_OVERLAY_MODEL)?;
    Ok(root)
}

fn needs_overlay_source(root: &Path) -> std::path::PathBuf {
    root.join("source")
}

fn needs_overlay_policy() -> Result<ServiceOverlay, Box<dyn std::error::Error>> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.items = Some(ItemSelector::parse("Items")?);
    plan.paginator = Some(Paginator::None);
    plan.identity = Some(Identity::Key(vec![MemberPath::selected("Id")?]));
    Ok(ServiceOverlay::new(
        "needs",
        vec![("ListNeedsOverlay", plan)],
    )?)
}

#[test]
fn paginator_none_rejected_when_model_is_modeled() -> Result<(), Box<dyn std::error::Error>> {
    let root = paginated_fixture()?;
    let policy = paginated_policy()?;
    match overlay::check_policies(
        &paginated_source(root.path()),
        &[policy],
        &DenyList::builtin(),
    ) {
        Err(CheckError::Violations(errors)) => {
            let has_invalid_value = errors.iter().any(|e| {
                e.rule() == ViolationRule::InvalidValue
                    && e.key() == "paginator"
                    && e.value() == "Paginator::None"
            });
            assert!(
                has_invalid_value,
                "expected InvalidValue violation for Paginator::None with modeled pagination; got {:?}",
                errors
            );
        }
        other => {
            return Err(format!("expected violations but got {:?}", other).into());
        }
    }
    Ok(())
}

#[test]
fn paginator_none_accepted_for_needs_overlay_pagination() -> Result<(), Box<dyn std::error::Error>>
{
    let root = needs_overlay_fixture()?;
    let policy = needs_overlay_policy()?;
    let checked = overlay::check_policies(
        &needs_overlay_source(root.path()),
        &[policy],
        &DenyList::builtin(),
    );
    // NeedsOverlay means the SDK handles pagination; Paginator::None is valid here.
    match &checked {
        Err(overlay::CheckError::Violations(errors)) => {
            for e in errors {
                eprintln!(
                    "violation: {} {} {} {}",
                    e.service(),
                    e.operation(),
                    e.key(),
                    e.value()
                );
            }
        }
        Err(other) => eprintln!("error: {:?}", other),
        _ => {}
    }
    assert!(
        checked.is_ok(),
        "Paginator::None should be accepted for NeedsOverlay pagination; got {:?}",
        checked.as_ref().err()
    );
    Ok(())
}
