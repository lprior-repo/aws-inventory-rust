use super::{decode_scope_key, work::transitions::encode_scope};
use inventory_core::{
    inventory::{AccountId, OperationId, Region, ScopeKey, ServiceId},
    record::ContentHash,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn scoped() -> Result<ScopeKey, Box<dyn std::error::Error>> {
    Ok(ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("ec2")?,
        OperationId::parse("DescribeVpcs")?,
        None,
    )?
    .with_branch("page".into())?
    .with_context(ContentHash::from_bytes([7; 32])))
}

#[test]
fn given_persisted_scope_when_decoded_then_operation_and_context_are_available_to_consumers()
-> TestResult {
    let decoded = decode_scope_key(&encode_scope(&scoped()?)?)?;
    assert_eq!(decoded.service().as_str(), "ec2");
    assert_eq!(decoded.operation().as_str(), "DescribeVpcs");
    assert_eq!(decoded.account().as_str(), "123456789012");
    assert_eq!(decoded.region().as_str(), "us-east-1");
    assert_eq!(decoded.branch(), Some("page"));
    assert_eq!(decoded.context(), Some(ContentHash::from_bytes([7; 32])));
    Ok(())
}

#[test]
fn given_truncated_or_extended_persisted_scope_when_decoded_then_corrupt_keys_are_rejected()
-> TestResult {
    let bytes = encode_scope(&scoped()?)?;
    (0..bytes.len()).try_for_each(|end| {
        let prefix = bytes.get(..end).ok_or("invalid test range")?;
        if end > bytes.len().saturating_sub(32) {
            assert!(decode_scope_key(prefix).is_err());
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    let mut extended = bytes;
    extended.push(0);
    assert!(decode_scope_key(&extended).is_err());
    assert!(decode_scope_key(&[]).is_err());
    Ok(())
}
