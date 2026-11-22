use super::*;

#[test]
fn literal_emulator_parent_fixtures_preserve_all_sanitized_properties_and_exact_architecture_edges()
-> TestResult {
    let fixture: Value = serde_json::to_value(toml::from_str::<toml::Table>(include_str!(
        "../../../../tests/fixtures/cloudcontrol-parents.toml"
    ))?)?;
    let (mut ctx, _receiver) = context(3)?;
    ctx.account = "000000000000".into();
    let catalog = crate::cloud_control::catalog()?;
    let plans = plans()?;
    let parents = fixture
        .get("parents")
        .and_then(Value::as_array)
        .ok_or("missing literal parents")?;
    let children = fixture
        .get("children")
        .and_then(Value::as_array)
        .ok_or("missing literal children")?;
    let pagination = fixture
        .get("pagination_children")
        .and_then(Value::as_array)
        .ok_or("missing pagination children")?;
    assert_eq!(children.len(), 72);
    assert_eq!(pagination.len(), 98);
    let text = |row: &Value, key: &str| -> Result<String, TestError> {
        row.get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("missing fixture field {key}").into())
    };
    let mut architecture_edges = 0_usize;
    for row in children.iter().chain(pagination) {
        let type_name = text(row, "type_name")?;
        let descriptor = find_type(&catalog, &type_name)?;
        let plan = plans
            .iter()
            .find(|plan| plan.type_name == type_name)
            .ok_or("missing fixture plan")?
            .clone();
        let child = ChildType { descriptor, plan };
        let parent_key = text(row, "parent_key")?;
        let parent_fixture = parents
            .iter()
            .find(|parent| parent.get("key").and_then(Value::as_str) == Some(parent_key.as_str()))
            .ok_or("missing fixture parent")?;
        let parent_id = parent_fixture
            .get("expected_resource_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("literal-parent:{parent_key}"));
        let parent_resource = Resource {
            id: parent_id.clone(),
            kind: text(parent_fixture, "expected_kind")?,
            account_id: ctx.account.clone(),
            region: ctx.region.clone(),
            name: None,
            tags: BTreeMap::new(),
            source: Source::CloudControl,
            observed_at: 10,
            properties: json!({"provider_properties":parent_fixture.get("provider").ok_or("missing parent provider")?}),
        };
        let parent = parent_context(&child, &parent_resource)?;
        assert_eq!(
            serde_json::to_value(&parent.model)?,
            *row.get("expected_model").ok_or("missing expected model")?
        );
        let id = text(row, "name")?;
        let provider = row.get("provider").ok_or("missing child provider")?.clone();
        let body = map_description(&ctx, descriptor, &id, &description(&id, provider), true)?;
        let body = attach_parent(body, &child, &parent, true)?;
        let resource = body.resources.first().ok_or("missing mapped child")?;
        assert_eq!(resource.kind, text(row, "expected_kind")?);
        assert_eq!(
            resource.properties.get("provider_properties"),
            row.get("expected_provider")
        );
        assert!(!serde_json::to_string(resource)?.contains("HIDDEN"));
        let expected = row
            .get("expected_relationships")
            .and_then(Value::as_array)
            .ok_or("missing expected relationships")?;
        assert_eq!(body.edges.len(), expected.len() + 1);
        let parent_edges: Vec<_> = body
            .edges
            .iter()
            .filter(|edge| edge.relation == "CHILD_OF")
            .collect();
        assert_eq!(parent_edges.len(), 1);
        assert_eq!(
            parent_edges.first().ok_or("missing parent edge")?.to_id,
            parent_id
        );
        for wanted in expected {
            let relation = text(wanted, "relation")?;
            let target = text(wanted, "target")?;
            let matches: Vec<_> = body
                .edges
                .iter()
                .filter(|edge| edge.relation == relation && edge.to_id == target)
                .collect();
            assert_eq!(matches.len(), 1, "{type_name}: {relation} -> {target}");
            let edge = matches.first().ok_or("missing architecture edge")?;
            assert_eq!(edge.properties.get("target_kind"), wanted.get("kind"));
            assert_eq!(edge.properties.get("property_paths"), wanted.get("paths"));
            assert_eq!(
                edge.properties.get("owner_resolution"),
                wanted.get("owner_resolution")
            );
            architecture_edges += 1;
        }
    }
    assert_eq!(architecture_edges, 24);
    Ok(())
}
