use super::*;
use crate::record::{Key, Value};
use std::collections::BTreeMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn record(text: &str, secret: &str) -> Result<Record, RecordError> {
    Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Text(text.to_owned())),
        (Key::Field(FieldId::new(2)?), Value::Text(secret.to_owned())),
    ])))
}
fn projection() -> Result<ContextProjectionPlan, RecordError> {
    let path = vec![FieldId::new(1)?].into_boxed_slice();
    Ok(ContextProjectionPlan { selected: vec![path.clone()].into_boxed_slice(), response: vec![path.clone()].into_boxed_slice(), request: vec![path].into_boxed_slice() })
}

#[test]
fn durable_restart_preserves_correlated_namespaces_and_all_ancestors() -> TestResult {
    // Given two nested discoveries with distinct request/response/selected values.
    let first = DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListServers")?, 0,
        &projection()?, &record("server", "SECRET_SELECTED")?, &record("response-server", "SECRET_RESPONSE")?, &record("request-server", "SECRET_REQUEST")?)?;
    let context = DiscoveryContext::empty().push(first)?;
    let second = DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListUsers")?, 3,
        &projection()?, &record("user", "SECRET_USER")?, &record("response-user", "SECRET_USER_RESPONSE")?, &record("request-user", "SECRET_USER_REQUEST")?)?;
    let context = context.push(second)?;
    // When a durable queue is reopened from its opaque binary context.
    let bytes = context.encode()?;
    let reopened = DiscoveryContext::decode(&bytes)?;
    // Then projected fields and grandparent namespaces survive, never unreviewed fields.
    assert_eq!(context, reopened);
    assert!(!bytes.windows(6).any(|window| window == b"SECRET"));
    let path = [FieldId::new(1)?];
    assert!(matches!(reopened.value("transfer", "ListServers", ContextSource::SelectedItem, &path)?, Some(ValueRef::Text("server"))));
    assert!(matches!(reopened.value("transfer", "ListUsers", ContextSource::Response, &path)?, Some(ValueRef::Text("response-user"))));
    assert!(matches!(reopened.value("transfer", "ListUsers", ContextSource::Request, &path)?, Some(ValueRef::Text("request-user"))));
    assert!(reopened.value("transfer", "ListUsers", ContextSource::SelectedItem, &[FieldId::new(2)?])?.is_none());
    Ok(())
}

#[test]
fn branch_discriminator_distinguishes_equal_selected_rows_by_parent_and_ordinal() -> TestResult {
    let make = |ordinal, parent| -> Result<DiscoveryContext, ContextError> {
        DiscoveryContext::empty().push(DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListUsers")?, ordinal,
            &projection()?, &record("same-user", "")?, &record(parent, "")?, &record("request", "")?)?)
    };
    assert_ne!(make(0, "server-a")?.discriminator()?, make(0, "server-b")?.discriminator()?);
    assert_ne!(make(0, "server-a")?.discriminator()?, make(1, "server-a")?.discriminator()?);
    Ok(())
}

#[test]
fn context_rejects_depth_and_encoded_byte_overflow() -> TestResult {
    let row = record("a", "")?;
    let frame = DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListUsers")?, 0, &projection()?, &row, &row, &row)?;
    let context = (0..MAX_DISCOVERY_FRAMES).try_fold(DiscoveryContext::empty(), |context, _| context.push(frame.clone()))?;
    assert!(matches!(context.push(frame), Err(ContextError::Bounds)));
    assert!(matches!(DiscoveryContext::decode(&vec![0; MAX_DISCOVERY_CONTEXT_BYTES.saturating_add(1)]), Err(ContextError::Bounds)));
    Ok(())
}

#[test]
fn nonallocating_wire_size_matches_canonical_encoding_across_header_boundaries() -> TestResult {
    assert_eq!(DiscoveryContext::empty().encoded_len()?, DiscoveryContext::empty().encode()?.len());
    for length in [0, 23, 24, 255, 256, 65_535, 65_536] {
        let row = record(&"x".repeat(length), "")?;
        for ordinal in [0, 23, 24, 255, 256, 65_535, 65_536, u32::MAX] {
            let frame = DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListUsers")?,
                ordinal, &projection()?, &row, &row, &row)?;
            let context = DiscoveryContext::empty();
            let predicted = context.pushed_encoded_len(&frame)?;
            let context = context.push(frame)?;
            assert_eq!(predicted, context.encode()?.len());
            assert_eq!(predicted, context.encoded_len()?);
        }
    }
    let row = record("small", "")?;
    let frame = DiscoveryFrame::project(ServiceId::parse("transfer")?, OperationId::parse("ListUsers")?, 0,
        &projection()?, &row, &row, &row)?;
    let mut context = DiscoveryContext::empty();
    for _ in 0..MAX_DISCOVERY_FRAMES {
        let predicted = context.pushed_encoded_len(&frame)?;
        context = context.push(frame.clone())?;
        assert_eq!(predicted, context.encode()?.len());
    }
    Ok(())
}

#[test]
fn given_same_operation_stages_when_context_reopens_then_stage_lookup_never_uses_another_stage() -> TestResult {
    let frame = |branch, selected, response, request| -> Result<DiscoveryFrame, ContextError> {
        DiscoveryFrame::project(ServiceId::parse("proton")?, OperationId::parse("ListTemplateVersions")?, 0,
            &projection()?, &record(selected, "SECRET")?, &record(response, "SECRET")?, &record(request, "SECRET")?)?
            .with_branch(Some(branch))
    };
    let major = frame("majors", "major", "major-response", "major-request")?;
    let minor = frame("minors", "minor", "minor-response", "minor-request")?;
    let context = DiscoveryContext::empty().push(major)?.push(minor)?;
    let encoded = context.encode()?;
    assert_eq!(context.encoded_len()?, encoded.len());
    let reopened = DiscoveryContext::decode(&encoded)?;
    assert_eq!(reopened, context);
    let path = [FieldId::new(1)?];
    for (source, expected) in [(ContextSource::SelectedItem, "major"), (ContextSource::Response, "major-response"), (ContextSource::Request, "major-request")] {
        assert!(matches!(reopened.value_from_branch("proton", "ListTemplateVersions", "majors", source, &path)?,
            Some(ValueRef::Text(value)) if value == expected));
        assert!(reopened.value_from_branch("proton", "ListTemplateVersions", "unknown", source, &path)?.is_none());
        assert!(reopened.value_from_branch("foreign", "ListTemplateVersions", "majors", source, &path)?.is_none());
    }
    assert!(matches!(reopened.value("proton", "ListTemplateVersions", ContextSource::SelectedItem, &path)?, Some(ValueRef::Text("minor"))));
    assert_ne!(DiscoveryContext::empty().push(frame("majors", "same", "same", "same")?)?.discriminator()?,
        DiscoveryContext::empty().push(frame("minors", "same", "same", "same")?)?.discriminator()?);
    Ok(())
}

#[test]
fn given_bounded_stage_names_when_frames_encode_then_exact_sizes_hold_and_invalid_names_are_rejected() -> TestResult {
    let row = record("value", "")?;
    let frame = DiscoveryFrame::project(ServiceId::parse("proton")?, OperationId::parse("ListTemplateVersions")?,
        u32::MAX, &projection()?, &row, &row, &row)?;
    for name in ["a".to_owned(), "a".repeat(23), "a".repeat(24), "a".repeat(64), "major_1-stage".to_owned()] {
        let frame = frame.clone().with_branch(Some(&name))?;
        let empty = DiscoveryContext::empty();
        let predicted = empty.pushed_encoded_len(&frame)?;
        let context = empty.push(frame)?;
        assert_eq!(context.encode()?.len(), predicted);
        assert_eq!(context.encoded_len()?, predicted);
    }
    for name in ["".to_owned(), "a".repeat(65), "major.stage".to_owned(), "major/stage".to_owned(), "é".to_owned()] {
        assert!(matches!(frame.clone().with_branch(Some(&name)), Err(ContextError::Invalid)));
    }
    Ok(())
}
