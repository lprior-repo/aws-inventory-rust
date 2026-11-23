use super::*;

#[test]
fn given_invalid_duration_when_parsing_then_configuration_is_rejected() {
    for value in ["0", "0.99", "-1", "601", "NaN", "inf", "not-seconds"] {
        assert!(FrameConfig::parse(value).is_err());
    }
    assert!(FrameConfig::parse("1").is_ok());
    assert!(FrameConfig::parse("600").is_ok());
    assert!(FrameConfig::parse("1.5").is_ok());
}

#[test]
fn given_sample_budget_when_full_then_capture_refuses_growth() -> Result<(), String> {
    let mut samples = reserve(MAX_SAMPLES)?;
    let sample = report::Sample {
        elapsed_ms: 0.0,
        ms: 1.0,
    };
    samples.resize(MAX_SAMPLES, sample);
    let capacity = samples.capacity();
    assert!(!push(&mut samples, sample));
    assert_eq!(samples.len(), MAX_SAMPLES);
    assert_eq!(samples.capacity(), capacity);
    Ok(())
}

#[test]
fn given_invalid_timing_when_recording_then_sample_is_rejected() {
    let mut samples = Vec::new();
    for ms in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(!push(
            &mut samples,
            report::Sample {
                elapsed_ms: 0.0,
                ms
            }
        ));
    }
    assert!(samples.is_empty());
}
