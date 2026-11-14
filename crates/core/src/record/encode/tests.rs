use super::{Bounded, MAX_RECORD_BYTES, RecordError, Write};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn oversized_chunk_is_rejected_without_copying_any_payload() -> TestResult {
    let mut writer = Bounded::default();
    writer.write_all(&[0xA1, 1])?;
    let payload = vec![0; MAX_RECORD_BYTES];

    assert_eq!(writer.write_all(&payload), Err(RecordError::SizeLimit));
    assert_eq!(writer.0, [0xA1, 1]);
    Ok(())
}

#[test]
fn aggregate_writes_stop_at_the_inclusive_limit() -> TestResult {
    let mut writer = Bounded::default();
    let chunk = vec![0; 1_048_576];
    (0..16).try_for_each(|_| writer.write_all(&chunk))?;
    assert_eq!(writer.0.len(), MAX_RECORD_BYTES);

    assert_eq!(writer.write_all(&chunk), Err(RecordError::SizeLimit));
    assert_eq!(writer.0.len(), MAX_RECORD_BYTES);
    Ok(())
}

#[test]
fn non_writer_encoder_failures_remain_distinct_from_size_failures() {
    let fault = minicbor::encode::Error::<RecordError>::message("encoding fault");

    assert_eq!(RecordError::from(fault), RecordError::Encode);
}
