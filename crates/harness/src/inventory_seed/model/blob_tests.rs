use super::*;
use inventory_core::record::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const HEADER: &str = "format=1\nservice='translate'\n[[step]]\nid='import'\noperation='ImportTerminology'\nkind=''\n[step.input]\nName='inventory-blob'\nMergeStrategy='OVERWRITE'\n";

fn bytes() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )
}

fn prepare(runtime: &RuntimeCatalog<'_>, encoded: &str) -> Result<Vec<PreparedInput>, SeedError> {
    let document = format!("{HEADER}TerminologyData={{File='{encoded}',Format='CSV'}}\n");
    let scenario = crate::inventory_seed::load(&document)?;
    SeedModels::load(runtime, &scenario)?
        .steps
        .into_iter()
        .next()
        .map(|step| step.inputs)
        .ok_or(SeedError::Invalid("test import step is missing"))
}

fn blob(inputs: &[PreparedInput]) -> Result<&[u8], Box<dyn std::error::Error>> {
    inputs
        .iter()
        .find_map(|input| match &input.value {
            PreparedInputValue::Structure(fields) => {
                fields.iter().find_map(|(_, value)| match value {
                    PreparedInputValue::Literal(Value::Bytes(bytes)) => Some(bytes.as_slice()),
                    _ => None,
                })
            }
            _ => None,
        })
        .ok_or_else(|| "prepared blob is missing".into())
}

#[test]
fn given_canonical_blob_strings_when_toml_prepared_then_padding_and_binary_bytes_decode()
-> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    [
        ("Zg==", b"f".as_slice()),
        ("Zm8=", b"fo"),
        ("Zm9v", b"foo"),
        ("AP8=", &[0, 255]),
    ]
    .into_iter()
    .try_for_each(|(encoded, expected)| -> TestResult {
        let inputs = prepare(&runtime, encoded)?;
        assert_eq!(blob(&inputs)?, expected);
        Ok(())
    })
}

#[test]
fn given_maximum_encoded_blob_when_toml_prepared_then_decoded_bytes_remain_bounded() -> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let encoded = "AAAA".repeat(256);
    assert_eq!(encoded.len(), 1024);
    let inputs = prepare(&runtime, &encoded)?;
    assert_eq!(blob(&inputs)?, vec![0; 768]);
    Ok(())
}

#[test]
fn given_noncanonical_blob_strings_when_toml_prepared_then_preflight_rejects_them() -> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    [
        "%bad", "Zg", "Zg=", "Zg===", "Zh==", "Zm9=", "Zg== ", "Z g==", "-w==", "__8=", "é===",
    ]
    .iter()
    .for_each(|encoded| {
        assert!(
            matches!(
                prepare(&runtime, encoded),
                Err(SeedError::Invalid(
                    "seed blob must be canonical padded base64"
                ))
            ),
            "accepted noncanonical blob {encoded:?}"
        );
    });
    Ok(())
}

#[test]
fn given_ordinary_text_beside_blob_when_toml_prepared_then_text_is_not_base64_decoded() -> TestResult
{
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let inputs = prepare(&runtime, "Zm9v")?;
    let text = inputs
        .iter()
        .filter_map(|input| match &input.value {
            PreparedInputValue::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(text, ["inventory-blob", "OVERWRITE"]);
    assert_eq!(blob(&inputs)?, b"foo");
    Ok(())
}

#[test]
fn given_oversized_blob_at_modeled_boundary_when_prepared_then_decode_is_rejected() -> TestResult {
    let bytes = bytes()?;
    let catalog = Catalog::borrow(&bytes)?;
    let operation = locate(catalog, "translate", "ImportTerminology")?;
    let value = SeedInputValue::Text("AAAA".repeat(257));
    let result = prepare_value(
        &[operation],
        operation,
        &FieldPath::new(&["TerminologyData", "File"])?,
        &value,
    );
    assert!(matches!(
        result,
        Err(SeedError::Invalid(
            "seed blob exceeds encoded length bound 1024"
        ))
    ));
    Ok(())
}

#[test]
fn given_empty_toml_blob_when_loaded_then_existing_nonempty_text_bound_rejects_it() -> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(matches!(
        prepare(&runtime, ""),
        Err(SeedError::Invalid("seed text exceeds its value bound"))
    ));
    Ok(())
}
