//! Given historical SDK outcomes, coverage never transfers them to another version.
use super::version_fixture::*;
use super::*;

#[test]
fn old_success_does_not_credit_same_module_in_new_active_version() -> TestResult {
    // Given: identical operation names in two versions, with only the old version executed.
    let directory = versioned_fixture("2.0.0")?;
    // When: the lockfile selects the new version.
    let report = operation_coverage(directory.path())?.to_value()?;
    // Then: both new alpha operations remain unexercised, without invalidating the source scan.
    assert_eq!(number(&report, "/summary/exercised")?, 3);
    assert_eq!(number(&report, "/summary/unexercised")?, 6);
    assert_scan_preserves_lexical_calls(&report)?;
    assert_eq!(array(&report, "/excluded_operations")?.len(), 0);
    let alpha: Vec<_> = array(&report, "/unexercised_sample")?
        .iter()
        .filter(|row| row.get("service") == Some(&json!("alpha")))
        .collect();
    assert_eq!(alpha.len(), 4);
    Ok(())
}

#[test]
fn matching_old_version_keeps_its_success_and_decode_failure() -> TestResult {
    // Given: the same mixed-version inventory and frozen old-version outcomes.
    let directory = versioned_fixture("1.0.0")?;
    // When: the lockfile selects the recorded version.
    let report = operation_coverage(directory.path())?.to_value()?;
    // Then: the success receives credit and its recorded decode failure is excluded.
    assert_eq!(number(&report, "/summary/exercised")?, 4);
    assert_eq!(array(&report, "/excluded_operations")?.len(), 1);
    assert_eq!(
        pointer(&report, "/excluded_operations/0/module")?,
        &json!("alpha_only")
    );
    assert_scan_preserves_lexical_calls(&report)?;
    Ok(())
}

#[test]
fn missing_historical_version_fails_closed_without_rebinding() -> TestResult {
    // Given: the old recorded version is no longer available in the published inventory.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    remove_old_inventory(root)?;
    // When/Then: coverage names the unavailable historical identity instead of using version 2.
    assert_eq!(
        failure(root)?,
        "recorded operation alpha.list_things at aws-sdk-alpha 1.0.0 is absent from catalog/aws-sdk-operations.json"
    );
    Ok(())
}

#[test]
fn unchanged_sources_do_not_make_old_sdk_receipt_current() -> TestResult {
    // Given: unchanged collector source and a receipt pinned to the old SDK.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    install_receipt(root, &receipt(root, "1.0.0")?)?;
    // When: coverage considers the receipt against the upgraded lockfile.
    let report = operation_coverage(root)?.to_value()?;
    // Then: the receipt is stale, and old success does not credit the new SDK.
    assert_eq!(number(&report, "/summary/exercised")?, 3);
    assert_eq!(
        pointer(&report, "/status_source")?,
        &json!("operation_matrix")
    );
    assert_eq!(
        pointer(&report, "/sources/reconciliation/status")?,
        &json!("stale_sdk_closure")
    );
    Ok(())
}

#[test]
fn current_version_receipt_credits_only_its_exact_bound_operation() -> TestResult {
    // Given: a new execution receipt bound to the active SDK version and operation module.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    install_receipt(root, &receipt(root, "2.0.0")?)?;
    // When: coverage applies that receipt.
    let report = operation_coverage(root)?.to_value()?;
    // Then: precisely the operation in the receipt is exercised.
    assert_eq!(number(&report, "/summary/exercised")?, 1);
    assert_eq!(
        pointer(&report, "/status_source")?,
        &json!("reconciliation")
    );
    assert_eq!(
        pointer(&report, "/exercised_operations/0/module")?,
        &json!("list_things")
    );
    assert_eq!(
        pointer(&report, "/sources/reconciliation/status")?,
        &json!("applied")
    );
    Ok(())
}

#[test]
fn current_version_receipt_with_historical_module_digest_is_not_applied() -> TestResult {
    // Given: current source and SDK version, but the historical operation module digest.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    let mut document = receipt(root, "2.0.0")?;
    set(
        &mut document,
        "/pinned_sdk_operation_modules/0/module_sha256",
        json!("a".repeat(64)),
    )?;
    install_receipt(root, &document)?;
    // When/Then: coverage refuses to treat the receipt as current evidence.
    let report = operation_coverage(root)?.to_value()?;
    assert_eq!(number(&report, "/summary/exercised")?, 3);
    assert_eq!(
        pointer(&report, "/sources/reconciliation/status")?,
        &json!("stale_sdk_closure")
    );
    Ok(())
}

#[test]
fn receipt_outcome_without_corresponding_sdk_pin_is_rejected() -> TestResult {
    // Given: the receipt claims an operation not covered by its SDK pin closure.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    let mut document = receipt(root, "2.0.0")?;
    set(
        &mut document,
        "/emulator_evidence/operation_results/0/id",
        json!("alpha.alpha_only"),
    )?;
    install_receipt(root, &document)?;
    // When/Then: the missing pin is an explicit provenance failure, not inferred from the lockfile.
    assert_eq!(
        failure(root)?,
        "reconciliation outcome alpha.alpha_only has no pinned SDK operation"
    );
    Ok(())
}

#[test]
fn source_change_keeps_receipt_stale_even_with_current_sdk_pins() -> TestResult {
    // Given: a receipt with current SDK pins but an obsolete collector source digest.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    let mut document = receipt(root, "2.0.0")?;
    set(
        &mut document,
        "/aws_source_files/0/sha256",
        json!("b".repeat(64)),
    )?;
    install_receipt(root, &document)?;
    // When/Then: only frozen matching-version matrix evidence remains applicable.
    let report = operation_coverage(root)?.to_value()?;
    assert_eq!(number(&report, "/summary/exercised")?, 3);
    assert_eq!(
        pointer(&report, "/sources/reconciliation/status")?,
        &json!("stale_source_closure")
    );
    Ok(())
}

#[test]
fn missing_recorded_sdk_version_cannot_inherit_active_version() -> TestResult {
    // Given: a versioned inventory whose matrix omits an operation's version.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    let mut document = read(root, MATRIX)?;
    document
        .pointer_mut("/operations/0")
        .and_then(Value::as_object_mut)
        .ok_or("matrix operation")?
        .remove("sdk_version");
    write(root, MATRIX, &document)?;
    // When/Then: coverage refuses to infer the active version from the lockfile.
    assert_eq!(
        failure(root)?,
        "recorded operation alpha.list_things at aws-sdk-alpha  is absent from catalog/aws-sdk-operations.json"
    );
    Ok(())
}

#[test]
fn receipt_without_sdk_pin_closure_is_rejected() -> TestResult {
    // Given: source closure alone cannot identify which SDK version executed.
    let directory = versioned_fixture("2.0.0")?;
    let root = directory.path();
    let mut document = receipt(root, "2.0.0")?;
    document
        .as_object_mut()
        .ok_or("receipt")?
        .remove("pinned_sdk_operation_modules");
    install_receipt(root, &document)?;
    // When/Then: missing pin metadata is an explicit invalid-input failure.
    assert_eq!(failure(root)?, "missing pinned_sdk_operation_modules");
    Ok(())
}
