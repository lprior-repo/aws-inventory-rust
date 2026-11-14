use super::*;
use crate::smithy::model::Shape;

#[test]
fn default_client_optional_and_mixin_payloads_are_retained() -> Result<(), String> {
    let shape: Shape = serde_json::from_str(r#"{"type":"string","traits":{"smithy.api#default":"AQI=","smithy.api#clientOptional":{},"smithy.api#mixin":{"localTraits":["example#local"]}}}"#).map_err(|error| error.to_string())?;
    assert_eq!(
        shape.traits.get("smithy.api#default"),
        Some(&serde_json::json!("AQI="))
    );
    assert_eq!(
        shape.traits.get("smithy.api#clientOptional"),
        Some(&serde_json::json!({}))
    );
    validate_shape(&shape)
}

#[test]
fn client_optional_rejects_malformed_values() -> Result<(), String> {
    let shape: Shape =
        serde_json::from_str(r#"{"type":"string","traits":{"smithy.api#clientOptional":"true"}}"#)
            .map_err(|error| error.to_string())?;
    assert_eq!(
        validate_shape(&shape),
        Err("smithy.api#clientOptional".into())
    );
    Ok(())
}
