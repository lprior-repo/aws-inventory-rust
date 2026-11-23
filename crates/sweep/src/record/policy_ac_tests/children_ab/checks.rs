use super::*;

pub(super) fn verify_catalog(
    catalog: &RuntimeCatalog<'_>,
    service: &str,
    kinds: &[(&str, &str)],
    excluded: &[&str],
) -> TestResult {
    for (operation, kind) in kinds {
        assert_eq!(
            catalog
                .lookup(service, operation)
                .ok_or("child plan absent")?
                .kind()
                .as_str(),
            *kind
        );
    }
    for operation in excluded {
        assert!(
            catalog.lookup(service, operation).is_none(),
            "excluded operation became executable: {operation}"
        );
    }
    Ok(())
}
