use super::{TestResult, compile_model, fixture_service};
use inventory_core::catalog::CompressionAlgorithm;
use serde_json::{Value, json};

const TRAIT: &str = "smithy.api#requestCompression";

fn model(trait_name: &str, payload: Value) -> Result<String, serde_json::Error> {
    let mut traits = serde_json::Map::new();
    traits.insert(trait_name.to_owned(), payload);
    serde_json::to_string(&json!({
        "smithy": "2.0",
        "shapes": {
            "example#Service": {
                "type": "service", "version": "2026-11-14",
                "operations": [{"target": "example#Compress"}],
                "traits": {"aws.api#service": {"sdkId": "Compression"}, "aws.protocols#awsQuery": {}}
            },
            "example#Compress": {"type": "operation", "traits": traits}
        }
    }))
}

fn algorithm(bytes: &[u8]) -> Result<Option<CompressionAlgorithm>, Box<dyn std::error::Error>> {
    let service = fixture_service(bytes)?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    Ok(operation.request_compression()?)
}

#[test]
fn modeled_compression_is_canonical_and_absent_trait_stays_absent() -> TestResult {
    let lower = model(TRAIT, json!({"encodings": ["gzip"]}))?;
    let mixed = model(TRAIT, json!({"encodings": ["GzIp"]}))?;
    let absent = model("other#requestCompression", json!({"encodings": ["gzip"]}))?;
    let (_source, lower_bytes) = compile_model(&lower, "")?;
    let (_source, mixed_bytes) = compile_model(&mixed, "")?;
    let (_source, absent_bytes) = compile_model(&absent, "")?;
    assert_eq!(algorithm(&lower_bytes)?, Some(CompressionAlgorithm::Gzip));
    assert_eq!(lower_bytes, mixed_bytes);
    assert_eq!(algorithm(&absent_bytes)?, None);
    Ok(())
}

#[test]
fn malformed_compression_traits_name_service_shape_and_trait() -> TestResult {
    for payload in [
        Value::Null,
        json!(true),
        json!({}),
        json!({"encodings": "gzip"}),
        json!({"encodings": []}),
        json!({"encodings": [1]}),
        json!({"encodings": ["zstd"]}),
        json!({"encodings": ["gzip", "zstd"]}),
    ] {
        let source = model(TRAIT, payload)?;
        let error = compile_model(&source, "")
            .err()
            .ok_or("malformed compression accepted")?;
        let message = error.to_string();
        assert!(message.contains("example"), "{message}");
        assert!(message.contains("example#Compress"), "{message}");
        assert!(message.contains(TRAIT), "{message}");
    }
    Ok(())
}
