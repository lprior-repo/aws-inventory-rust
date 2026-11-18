use super::*;
use std::collections::BTreeMap;

#[test]
fn resolver_enforces_step_limit_on_a_real_mixin_edge() -> Result<(), String> {
    let source = r#"{"shapes":{"n#Base":{"type":"structure","traits":{"smithy.api#mixin":{}}},"n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]}}}"#;
    let model = parse(source)?;
    let mut resolved = BTreeMap::new();
    let mut steps = MAX_MIXIN_STEPS - 1;
    let mut bytes = 0;
    let result = resolve("n#Input", &model, &mut resolved, &mut steps, &mut bytes);
    assert_eq!(
        result.err().as_deref(),
        Some("mixin traversal exceeds 1000000 steps")
    );
    assert_eq!(steps, MAX_MIXIN_STEPS);
    Ok(())
}

#[test]
fn expansion_stops_at_estimated_inherited_byte_budget() -> Result<(), String> {
    let model = fibonacci_mixin_graph(32)?;
    assert_eq!(
        expand(model).err().as_deref(),
        Some("mixin expansion exceeds 268435456 bytes")
    );
    Ok(())
}

fn fibonacci_mixin_graph(levels: usize) -> Result<ModelFile, String> {
    let mut shapes = serde_json::Map::new();
    let payload = "x".repeat(1024);
    for level in 0..levels {
        let mut shape = serde_json::json!({"type":"structure","traits":{"smithy.api#mixin":{}},"members":{"value":{"target":"smithy.api#String","traits":{"smithy.api#default":payload.clone()}}}});
        if level == 1 {
            shape["mixins"] = serde_json::json!([{"target":"n#M00"}]);
        } else if level > 1 {
            let level1 = level.checked_sub(1).unwrap_or(0);
    let level2 = level.checked_sub(2).unwrap_or(0);
    shape["mixins"] = serde_json::json!([{"target":format!("n#M{:02}", level1)},{"target":format!("n#M{:02}", level2)}]);
        }
        shapes.insert(format!("n#M{level:02}"), shape);
    }
    parse(&serde_json::json!({"shapes":shapes}).to_string())
}
