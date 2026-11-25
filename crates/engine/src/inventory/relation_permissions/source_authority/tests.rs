use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_reviewed_relation_when_hashed_then_legacy_input_bytes_are_unchanged() -> TestResult {
    let legacy = concat!(
        "\x0e\0\0\0AWS::Oam::Link",
        "\0\x01\0\0\0\x01\0\0\0",
        "\x0e\0\0\0AWS::Oam::Sink",
        "\x01\0\0\0\x0e\0\0\0AWS::Oam::Sink",
        "\x01\0\0\0\x02\0\0\0",
        "\x0a\0\0\0REFERENCES\0\0",
    );
    let rule = super::super::tests::declared_rule()?;
    let mut hash = Sha256::new();
    relation(&mut hash, &rule)?;
    let actual: [u8; 32] = hash.finalize().into();
    let expected: [u8; 32] = Sha256::digest(legacy.as_bytes()).into();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn given_identical_relations_when_provenance_changes_then_authority_digests_differ() -> TestResult {
    let mut rule = super::super::tests::declared_rule()?;
    let mut reviewed = Sha256::new();
    relation(&mut reviewed, &rule)?;
    rule.edge_source = inventory_core::inventory::EdgeSource::Model;
    let mut model = Sha256::new();
    relation(&mut model, &rule)?;
    let reviewed: [u8; 32] = reviewed.finalize().into();
    let model: [u8; 32] = model.finalize().into();
    assert_ne!(reviewed, model);
    Ok(())
}
