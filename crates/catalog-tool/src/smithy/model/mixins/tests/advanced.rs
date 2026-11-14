use super::*;

#[test]
fn intermediate_local_traits_apply_to_transitive_inheritance() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":["smithy.api#xmlName"]},"smithy.api#xmlName":"base","smithy.api#xmlNamespace":{"uri":"urn:base"}},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#default":"base"}}}},
        "n#Middle":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":["smithy.api#xmlNamespace"]},"smithy.api#xmlName":"middle","smithy.api#xmlNamespace":{"uri":"urn:middle"}},"mixins":[{"target":"n#Base"}],"members":{}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Middle"}],"members":{}}
    }}"#;
    let model = expand(parse(source)?)?;
    let input = model.shapes.get("n#Input").ok_or("input")?;
    assert_eq!(
        input.traits.get("smithy.api#xmlName"),
        Some(&serde_json::json!("middle"))
    );
    assert_eq!(input.traits.get("smithy.api#xmlNamespace"), None);
    let ShapeKind::Structure { members } = &input.kind else {
        return Err("structure".into());
    };
    let member = members.get("value").ok_or("member")?;
    assert_eq!(member.target.as_deref(), Some("smithy.api#String"));
    assert_eq!(
        member.traits.get("smithy.api#default"),
        Some(&serde_json::json!("base"))
    );
    assert!(member.traits.contains_key("smithy.api#required"));
    Ok(())
}

#[test]
fn ordered_mixins_merge_same_target_member_traits() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#First":{"type":"structure","traits":{"smithy.api#mixin":{},"smithy.api#xmlName":"first"},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#default":"first"}}}},
        "n#Later":{"type":"structure","traits":{"smithy.api#mixin":{},"smithy.api#xmlName":"later"},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#jsonName":"wireValue","smithy.api#default":"later"}}}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#First"},{"target":"n#Later"}],"members":{}}
    }}"#;
    let model = expand(parse(source)?)?;
    let input = model.shapes.get("n#Input").ok_or("input")?;
    assert_eq!(
        input.traits.get("smithy.api#xmlName"),
        Some(&serde_json::json!("later"))
    );
    let ShapeKind::Structure { members } = &input.kind else {
        return Err("structure".into());
    };
    let traits = &members.get("value").ok_or("member")?.traits;
    assert_eq!(
        traits.get("smithy.api#default"),
        Some(&serde_json::json!("later"))
    );
    assert_eq!(
        traits.get("smithy.api#jsonName"),
        Some(&serde_json::json!("wireValue"))
    );
    assert!(traits.contains_key("smithy.api#required"));
    Ok(())
}

#[test]
fn ordered_mixins_reject_conflicting_member_targets() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#Text":{"type":"structure","traits":{"smithy.api#mixin":{}},"members":{"value":{"target":"smithy.api#String"}}},
        "n#Number":{"type":"structure","traits":{"smithy.api#mixin":{}},"members":{"value":{"target":"smithy.api#Integer"}}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Text"},{"target":"n#Number"}]}
    }}"#;
    assert_eq!(
        expand(parse(source)?).err().as_deref(),
        Some("conflicting mixin member targets: value")
    );
    Ok(())
}

#[test]
fn custom_local_trait_is_not_inherited() -> Result<(), String> {
    let source = r#"{"shapes":{
        "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{"localTraits":["example#localOnly"]},"example#localOnly":{"value":"payload"}}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]}
    }}"#;
    let model = expand(parse(source)?)?;
    assert!(
        model
            .shapes
            .get("n#Base")
            .is_some_and(|shape| shape.traits.contains_key("example#localOnly"))
    );
    assert!(
        !model
            .shapes
            .get("n#Input")
            .is_some_and(|shape| shape.traits.contains_key("example#localOnly"))
    );
    Ok(())
}

#[test]
fn direct_self_cycle_returns_its_shape_id() -> Result<(), String> {
    let source = r#"{"shapes":{"n#Self":{"type":"structure","traits":{"smithy.api#mixin":{}},"mixins":[{"target":"n#Self"}]}}}"#;
    assert_eq!(
        expand(parse(source)?).err().as_deref(),
        Some("mixin cycle includes n#Self")
    );
    Ok(())
}
