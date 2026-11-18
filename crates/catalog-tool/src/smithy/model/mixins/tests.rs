use super::*;

fn parse(source: &str) -> Result<ModelFile, String> {
    serde_json::from_str(source).map_err(|error| error.to_string())
}

#[test]
fn mixin_inherits_members_and_nonlocal_traits_with_direct_trait_precedence() -> Result<(), String> {
    assert_inheritance()
}

fn assert_inheritance() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":["smithy.api#xmlName"]},"smithy.api#xmlName":"base","smithy.api#xmlNamespace":{"uri":"urn:base"}},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#default":"base"}}}},
        "n#Intermediate":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":["smithy.api#xmlNamespace"]},"smithy.api#xmlName":"intermediate","smithy.api#xmlNamespace":{"uri":"urn:intermediate"}},"mixins":[{"target":"n#Base"}],"members":{}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Intermediate"}],"traits":{"smithy.api#xmlName":"input"},"members":{}}
    }}"#;
    let model = expand(parse(source)?)?;
    let input = model.shapes.get("n#Input").ok_or("input")?;
    assert_inherited_shape_traits(&model, input)?;
    let ShapeKind::Structure { members } = &input.kind else {
        return Err("structure".into());
    };
    let member = members.get("value").ok_or("member")?;
    assert_eq!(member.target.as_deref(), Some("smithy.api#String"));
    assert!(member.traits.contains_key("smithy.api#required"));
    assert_eq!(
        member.traits.get("smithy.api#default"),
        Some(&serde_json::json!("base"))
    );
    Ok(())
}

fn assert_inherited_shape_traits(model: &ModelFile, input: &Shape) -> Result<(), String> {
    assert_eq!(input.traits.get("smithy.api#mixin"), None);
    assert_eq!(
        input.traits.get("smithy.api#xmlName"),
        Some(&serde_json::json!("input"))
    );
    assert_eq!(input.traits.get("smithy.api#xmlNamespace"), None);
    assert_eq!(
        model
            .shapes
            .get("n#Base")
            .and_then(|shape| shape.traits.get("smithy.api#xmlName")),
        Some(&serde_json::json!("base"))
    );
    assert_eq!(
        model
            .shapes
            .get("n#Intermediate")
            .and_then(|shape| shape.traits.get("smithy.api#xmlName")),
        Some(&serde_json::json!("intermediate"))
    );
    Ok(())
}

#[test]
fn local_member_redefinition_merges_traits_when_targets_match() -> Result<(), String> {
    assert_member_override()
}

fn assert_member_override() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{}},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#default":"base"}}}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}],"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#jsonName":"localValue","smithy.api#default":"local"}}}}
    }}"#;
    let model = expand(parse(source)?)?;
    let ShapeKind::Structure { members } = &model.shapes.get("n#Input").ok_or("input")?.kind else {
        return Err("structure".into());
    };
    let traits = &members.get("value").ok_or("member")?.traits;
    assert!(traits.contains_key("smithy.api#required"));
    assert_eq!(
        traits.get("smithy.api#jsonName"),
        Some(&serde_json::json!("localValue"))
    );
    assert_eq!(
        traits.get("smithy.api#default"),
        Some(&serde_json::json!("local"))
    );
    Ok(())
}

#[test]
fn malformed_mixin_graphs_return_specific_errors() -> Result<(), String> {
    missing_target()?;
    non_mixin_target()?;
    mismatched_kind()?;
    malformed_marker()?;
    cyclic_graph()
}

fn missing_target() -> Result<(), String> {
    let model =
        parse(r#"{"shapes":{"n#Input":{"type":"structure","mixins":[{"target":"n#Missing"}]}}}"#)?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin target is missing: n#Missing")
    );
    Ok(())
}

fn non_mixin_target() -> Result<(), String> {
    let model = parse(
        r#"{"shapes":{"n#Base":{"type":"structure"},"n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]}}}"#,
    )?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin target is not a mixin: n#Base")
    );
    Ok(())
}

fn mismatched_kind() -> Result<(), String> {
    let model = parse(
        r#"{"shapes":{"n#Base":{"type":"union","traits":{"smithy.api#mixin":{}}},"n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]}}}"#,
    )?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin shape kind differs: n#Input")
    );
    Ok(())
}

fn malformed_marker() -> Result<(), String> {
    let model =
        parse(r#"{"shapes":{"n#Bad":{"type":"structure","traits":{"smithy.api#mixin":true}}}}"#)?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin trait must be an object")
    );
    let model = parse(
        r#"{"shapes":{"n#Bad":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":"example#private"}}}}}"#,
    )?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin localTraits must be an array")
    );
    let model = parse(
        r#"{"shapes":{"n#Bad":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":[1]}}}}}"#,
    )?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin localTraits entries must be strings")
    );
    Ok(())
}

fn cyclic_graph() -> Result<(), String> {
    let source = r#"{"shapes":{"n#A":{"type":"structure","traits":{"smithy.api#mixin":{}},"mixins":[{"target":"n#B"}]},"n#B":{"type":"structure","traits":{"smithy.api#mixin":{}},"mixins":[{"target":"n#A"}]}}}"#;
    assert_eq!(
        expand(parse(source)?).err().as_deref(),
        Some("mixin cycle includes n#A")
    );
    Ok(())
}

#[test]
fn mixin_depth_is_bounded() -> Result<(), String> {
    let model = deep_graph()?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin depth exceeds 128: n#129")
    );
    Ok(())
}

fn deep_graph() -> Result<ModelFile, String> {
    let mut shapes = serde_json::Map::new();
    for index in 0..=MAX_MIXIN_DEPTH + 1 {
        let mut shape = serde_json::json!({"type":"structure","traits":{"smithy.api#mixin":{}}});
        if index < MAX_MIXIN_DEPTH + 1 {
            let target = index.checked_add(1).unwrap_or(0);
    shape["mixins"] = serde_json::json!([{"target": format!("n#{}", target)}]);
        }
        shapes.insert(format!("n#{index}"), shape);
    }
    parse(&serde_json::json!({"shapes": shapes}).to_string())
}
#[path = "tests/advanced.rs"]
mod advanced;
#[path = "tests/budget.rs"]
mod budget;
