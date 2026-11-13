use super::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 64,
        rng_seed: RngSeed::Fixed(0x4157_5349_4e56_0002),
        ..ProptestConfig::default()
    })]

    #[test]
    fn planning_deduplicates_targets_and_reports_a_cartesian_product(
        profiles in prop::collection::vec(0u8..4, 1..6),
        regions in prop::collection::vec(0u8..3, 1..6),
        resources in prop::collection::vec(0u8..3, 1..6),
    ) {
        let profile_names: Vec<String> = profiles.iter().map(|number| format!("fixture-profile-{number}")).collect();
        let region_names: Vec<&str> = regions.iter().map(|number| match number { 0 => "us-east-1", 1 => "us-west-2", _ => "eu-west-1" }).collect();
        let type_names: Vec<&str> = resources.iter().map(|number| match number { 0 => "AWS::EC2::VPC", 1 => "AWS::DynamoDB::Table", _ => "AWS::Lambda::Function" }).collect();
        let expected = profiles.iter().collect::<BTreeSet<_>>().len()
            .saturating_mul(regions.iter().collect::<BTreeSet<_>>().len())
            .saturating_mul(resources.iter().collect::<BTreeSet<_>>().len());
        let args = arguments(&["plan", "--cloud-control", "--service", "cloud-control", "--profile", &profile_names.join(","), "--region", &region_names.join(","), "--resource-type", &type_names.join(",")]);
        let plan = property(json_command(&args, 0))?;
        prop_assert_eq!(plan.pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"), Some(&json!(expected)));
        prop_assert_eq!(plan.pointer("/request_accounting/total_requests"), Some(&Value::Null));
        prop_assert_eq!(plan.pointer("/coverage/direct_resource_kinds"), Some(&json!(0)));
        prop_assert_eq!(plan.pointer("/coverage/live_validated"), Some(&json!(false)));
        prop_assert_eq!(plan.pointer("/targets/profile_count"), Some(&json!(profiles.iter().collect::<BTreeSet<_>>().len())));
    }

    #[test]
    fn query_literals_round_trip_without_becoming_sql_statements(value in "[a-zA-Z0-9 ;_':/.-]{0,80}") {
        let directory = tempfile::tempdir()?;
        let database = property(path(&directory.path().join("inventory.sqlite")))?;
        let input = property(fixture("global-events-before.jsonl"))?;
        property(json_command(&arguments(&["ingest", "--db", &database, "--scan-id", "literal-property", "--input", &input]), 0))?;
        let sql = format!("SELECT '{}' AS value", value.replace('\'', "''"));
        let rows = property(json_command(&arguments(&["query", "--db", &database, "--limit", "1", &sql]), 0))?;
        prop_assert_eq!(rows, json!([{"value":value}]));
        prop_assert!(property(command(&arguments(&["query", "--db", &database, "DELETE FROM scans"]), 1))?.is_empty());
    }

    #[test]
    fn query_row_limit_bounds_a_persisted_complete_fixture(limit in 1usize..9) {
        let directory = tempfile::tempdir()?;
        let database = property(path(&directory.path().join("inventory.sqlite")))?;
        let input = property(fixture("global-events-before.jsonl"))?;
        let ingest = property(json_command(&arguments(&["ingest", "--db", &database, "--scan-id", "property-snapshot", "--input", &input]), 0))?;
        prop_assert_eq!(ingest.get("status"), Some(&json!("complete")));
        if limit < 7 {
            prop_assert!(property(command(&arguments(&["query", "--db", &database, "--limit", &limit.to_string(), "SELECT id FROM latest_resources ORDER BY id"]), 1))?.is_empty());
        }
        let sql = format!("SELECT id FROM latest_resources ORDER BY id LIMIT {limit}");
        let rows = property(json_command(&arguments(&["query", "--db", &database, "--limit", &limit.to_string(), &sql]), 0))?;
        let rows = rows.as_array().ok_or_else(|| TestCaseError::fail("query did not return an array"))?;
        prop_assert_eq!(rows.len(), limit.min(7));
        prop_assert!(rows.iter().all(|row| row.get("id").and_then(Value::as_str).is_some_and(|id| !id.is_empty())));
    }
}
