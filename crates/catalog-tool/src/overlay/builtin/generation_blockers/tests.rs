use super::*;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_explicit_binding_blockers_when_loaded_then_every_migration_marks_an_existing_defer()
-> Result {
    let policies = crate::overlay::embedded()?;
    let mut expected = 0_usize;
    for (service, operations) in REQUIRED_BINDINGS {
        let policy = policies
            .iter()
            .find(|policy| policy.service() == *service)
            .ok_or("service missing")?;
        for operation in *operations {
            let entry = policy
                .operations
                .get(*operation)
                .ok_or("operation missing")?;
            assert_eq!(entry.class, Some(OverlayClass::Defer));
            assert_eq!(
                entry.generation_blocker,
                Some(GenerationBlocker::RequiredBindings)
            );
            expected = expected.checked_add(1).ok_or("count overflow")?;
        }
    }
    let actual = policies
        .iter()
        .flat_map(|policy| policy.operations.values())
        .filter(|entry| entry.generation_blocker.is_some())
        .count();
    assert_eq!(actual, expected);
    Ok(())
}
