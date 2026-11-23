use catalog_tool::{
    kind_map::{self, ApiOnlyMapping, KindMap, KindMapError, MappingSource, OperationMapping,
        generate::{Candidate, generate_candidates}},
    names::Exceptions,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CFN: &[u8] = br#"{"resource_types":[{"type_name":"AWS::Example::Widget"}]}"#;

fn given_hand() -> Result<KindMap> {
    Ok(MappingSource::default().compile(&Exceptions::none())?)
}

fn given_candidate<'a>(operation: &'a str, shape_id: &'a str) -> Candidate<'a> {
    Candidate {
        service: "example", operation, shape_id, resource: "Widget", sdk_id: "Example Service",
        cfn_service: Some("Example"), display: Some("Name"),
    }
}

#[test]
fn kind_map_given_exact_type_when_generated_then_cfn_and_display_are_emitted() -> Result {
    let generated = generate_candidates(vec![given_candidate("ListWidgets", "n#WidgetSummary")], &given_hand()?, CFN)?;
    assert_eq!(generated.text, "cfn\tAWS::Example::Widget\tmap\tAWS::Example::Widget\tName\noperation\texample\tListWidgets\tAWS::Example::Widget\n");
    assert!(generated.conflicts.is_empty());
    Ok(())
}

#[test]
fn kind_map_given_near_miss_when_generated_then_api_kind_keeps_display() -> Result {
    let mut candidate = given_candidate("ListWidgets", "n#Widget");
    candidate.cfn_service = Some("EXAMPLE");
    let generated = generate_candidates(vec![candidate], &given_hand()?, CFN)?;
    let map = given_hand()?.with_generated(generated.text.as_bytes(), &Exceptions::none())?;
    let row = map.api_only().first().ok_or("missing API-only row")?;
    assert_eq!(row.kind, "AWS::ApiOnly::ExampleServiceWidget");
    assert_eq!(row.display_name_path.as_deref(), Some("Name"));
    assert!(generated.text.contains("generated: no exact CloudFormation type"));
    Ok(())
}

#[test]
fn kind_map_given_different_shapes_when_generated_then_second_operation_is_deferred() -> Result {
    let generated = generate_candidates(vec![
        given_candidate("ListWidgets", "n#WidgetSummary"),
        given_candidate("ListOtherWidgets", "n#WidgetDescription"),
    ], &given_hand()?, CFN)?;
    assert_eq!(generated.conflicts.into_iter().collect::<Vec<_>>(), [("example".to_owned(), "ListWidgets".to_owned())]);
    assert!(generated.text.contains("operation\texample\tListOtherWidgets\t"));
    assert!(!generated.text.contains("operation\texample\tListWidgets\t"));
    Ok(())
}

#[test]
fn kind_map_given_same_shape_when_generated_twice_then_shared_kind_and_bytes_are_stable() -> Result {
    let candidates = || vec![given_candidate("ListWidgets", "n#Widget"), given_candidate("ListOtherWidgets", "n#Widget")];
    let first = generate_candidates(candidates(), &given_hand()?, CFN)?;
    let mut reversed = candidates();
    reversed.reverse();
    let second = generate_candidates(reversed, &given_hand()?, CFN)?;
    assert_eq!(first.text, second.text);
    assert_eq!(first.text.lines().filter(|line| line.starts_with("cfn\t")).count(), 1);
    assert_eq!(first.text.lines().filter(|line| line.starts_with("operation\t")).count(), 2);
    assert!(first.conflicts.is_empty());
    Ok(())
}

#[test]
fn kind_map_given_hand_keys_when_generated_rows_overlap_then_hand_wins() -> Result {
    let source = MappingSource {
        api_only: &[ApiOnlyMapping { kind: "AWS::ApiOnly::Original", reason: "Reviewed source" }],
        operations: &[OperationMapping { service: "example", operation: "ListWidgets", kind: "AWS::ApiOnly::Original" }],
        ..MappingSource::default()
    };
    let hand = source.compile(&Exceptions::none())?;
    let generated = b"api\tAWS::ApiOnly::Original\tkind\tGenerated replacement\tName\noperation\texample\tListWidgets\tAWS::Example::Widget\n";
    assert_eq!(hand.clone().with_generated(generated, &Exceptions::none())?, hand);
    Ok(())
}

#[test]
fn kind_map_given_generated_authority_rows_when_loaded_then_they_are_rejected() -> Result {
    for row in ["meta\t1", "relation\tAWS::Example::Widget", "bootstrap\texample", "default-unmapped\treason"] {
        assert!(matches!(given_hand()?.with_generated(row.as_bytes(), &Exceptions::none()), Err(KindMapError::InvalidRow { .. })));
    }
    Ok(())
}

#[test]
fn kind_map_given_colliding_names_when_loaded_then_existing_collision_check_rejects_them() -> Result {
    let rows = b"api\tAWS::ApiOnly::FooBar\tkind\tGenerated resource\napi\tAWS::ApiOnly::Foo::Bar\tkind\tGenerated resource\n";
    assert!(matches!(given_hand()?.with_generated(rows, &Exceptions::none()), Err(KindMapError::Duplicate(_))));
    Ok(())
}

#[test]
fn kind_map_given_invalid_generated_rows_when_loaded_then_validation_is_not_bypassed() -> Result {
    for bytes in [b"api\tAWS::ApiOnly::Widget\tkind\tValid reason\tbad..path\n".as_slice(),
        b"operation\texample\tListWidgets\tAWS::ApiOnly::Unknown\n",
        b"api\tAWS::ApiOnly::Widget\tkind\tValid reason\napi\tAWS::ApiOnly::Widget\tkind\tValid reason\n",
        b"\xff"] {
        assert!(given_hand()?.with_generated(bytes, &Exceptions::none()).is_err());
    }
    assert!(matches!(given_hand()?.with_generated(&vec![b'a'; 2_097_153], &Exceptions::none()), Err(KindMapError::TooLarge)));
    Ok(())
}

#[test]
fn kind_map_given_empty_generated_input_when_loaded_then_embedded_fingerprint_is_unchanged() -> Result {
    let source = include_bytes!("../../../catalog/cloudcontrol-universal.json");
    assert_eq!(kind_map::embedded(source)?, kind_map::load_with_cfn(source, b"")?);
    Ok(())
}
