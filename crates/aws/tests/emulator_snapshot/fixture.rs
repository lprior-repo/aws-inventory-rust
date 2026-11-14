use super::*;

pub(super) fn verified_snapshot() -> TestResult<Snapshot> {
    verify_environment()?;
    let bytes = bounded_file(
        &PathBuf::from(required_env("INVENTORY_EMULATOR_SNAPSHOT_PATH")?),
        MAX_SNAPSHOT_BYTES,
    )?;
    verify_digest(
        &bytes,
        &required_env("INVENTORY_EMULATOR_SNAPSHOT_SHA256")?,
        "snapshot",
    )?;
    let snapshot: Snapshot = serde_json::from_slice(&bytes)?;
    if snapshot.schema_version != 1
        || snapshot.account_id != ACCOUNT
        || snapshot.region != REGION
        || snapshot.provenance.emulator_version != VERSION
        || snapshot.provenance.emulator_commit != COMMIT
    {
        return Err(fail(
            "snapshot version, emulator pin, account or region does not match the guarded contract",
        ));
    }
    let summary_bytes = bounded_file(
        &PathBuf::from(required_env("INVENTORY_EMULATOR_BASELINE_SUMMARY")?),
        MAX_SNAPSHOT_BYTES,
    )?;
    verify_digest(
        &summary_bytes,
        &snapshot.provenance.summary_sha256,
        "baseline summary",
    )?;
    let summary: Value = serde_json::from_slice(&summary_bytes)?;
    let checks = summary
        .get("checks")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("baseline has no independent checks"))?;
    let names: BTreeSet<&str> = checks
        .iter()
        .filter_map(|check| check.get("check").and_then(Value::as_str))
        .collect();
    for required in [
        "endpoint:emulator_identity",
        "native:terminal_scan",
        "sqlite:integrity",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
        "coverage:every_registered_type_read",
    ] {
        if !names.contains(required) {
            return Err(fail(format!(
                "baseline lacks required independent assertion {required}"
            )));
        }
    }

    if !names.contains("global:all_35_native_kinds")
        && !names.contains("global:all_37_native_kinds")
        && !names.contains("global:all_39_native_kinds")
    {
        return Err(fail(
            "baseline lacks the all-native-kinds independent assertion",
        ));
    }
    if summary.get("failed_checks").and_then(Value::as_u64) != Some(0)
        || summary.get("passed_checks").and_then(Value::as_u64)
            != Some(u64::try_from(checks.len())?)
        || !checks
            .iter()
            .all(|check| check.get("passed").and_then(Value::as_bool) == Some(true))
        || summary
            .pointer("/emulator/source_commit")
            .and_then(Value::as_str)
            != Some(COMMIT)
        || summary.pointer("/emulator/account").and_then(Value::as_str) != Some(ACCOUNT)
        || summary.pointer("/emulator/region").and_then(Value::as_str) != Some(REGION)
    {
        return Err(fail(
            "baseline has failed, incomplete or mismatched independent verification",
        ));
    }
    let names: BTreeSet<&str> = snapshot
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect();
    if snapshot.cases.len() != 5
        || names
            != BTreeSet::from([
                "native",
                "sources",
                "cloudcontrol",
                "assumedrole",
                "parents",
            ])
    {
        return Err(fail(
            "snapshot must contain exactly the five required collector scenarios",
        ));
    }
    for case in &snapshot.cases {
        if !matches!(case.status.as_str(), "complete" | "partial")
            || case.resources.is_empty()
            || case.scopes.is_empty()
        {
            return Err(fail(
                "snapshot case lacks populated finalized baseline evidence",
            ));
        }
        for rows in [&case.resources, &case.edges, &case.scopes, &case.failures] {
            if rows.len() > MAX_ROWS || !rows.iter().all(Value::is_object) {
                return Err(fail("snapshot rows exceed their budget or are not objects"));
            }
        }
        if case
            .resources
            .iter()
            .chain(&case.edges)
            .any(|row| row.get("observed_at").is_some())
        {
            return Err(fail(
                "baseline must remove only the top-level local observation timestamp",
            ));
        }
    }
    if !snapshot
        .assume_role_arn
        .starts_with("arn:aws:iam::000000000000:role/")
    {
        return Err(fail("assumed-role scenario must use the fixture account"));
    }
    Ok(snapshot)
}

pub(super) fn options(name: &str, snapshot: &Snapshot) -> TestResult<AwsScanOptions> {
    let services = match name {
        "native" => vec!["all".into()],
        "sources" => vec!["config".into(), "resource-explorer".into()],
        "cloudcontrol" | "parents" => vec!["cloud-control".into()],
        "assumedrole" => vec!["iam".into()],
        _ => return Err(fail("unknown snapshot scenario")),
    };
    Ok(AwsScanOptions {
        profile: None,
        regions: vec![REGION.into()],
        services,
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: name == "sources",
        assume_role: (name == "assumedrole").then(|| AssumeRoleOptions {
            role_arn: snapshot.assume_role_arn.clone(),
            session_name: "aws-inventory".into(),
            external_id: None,
            duration_seconds: 3_600,
        }),
        cloud_control: matches!(name, "cloudcontrol" | "parents"),
        selection: inventory_core::Selection::Types(match name {
            "cloudcontrol" => inventory_aws::cloud_control::root_catalog()?
                .into_iter()
                .map(|entry| inventory_core::ResourceType::new(entry.type_name))
                .collect(),
            "parents" => inventory_aws::cloud_control::parent_catalog()?
                .into_iter()
                .map(|entry| inventory_core::ResourceType::new(entry.type_name))
                .collect(),
            _ => Vec::new(),
        }),
        max_pages: 128,
        page_size: 1,
    })
}

pub(super) fn now_millis() -> TestResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}
