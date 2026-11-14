include!("smithy_helpers.rs");

use catalog_tool::smithy::{Error, ReportMode, build_report, run};

#[test]
fn no_service_has_its_own_filename_bearing_variant() -> Result<(), Box<dyn std::error::Error>> {
    // Given a model with no service, when loaded, then its exact filename is retained.
    let dir = tempfile::tempdir()?;
    write_fixture(dir.path(), "models/empty.json", r#"{"shapes":{}}"#)?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing NoService error")?;
    match error {
        Error::NoService(file) => assert_eq!(file, dir.path().join("models/empty.json")),
        other => return Err(format!("expected NoService, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn several_services_have_their_own_filename_bearing_variant()
-> Result<(), Box<dyn std::error::Error>> {
    // Given two services, when loaded, then ambiguity is not conflated with no service.
    let dir = tempfile::tempdir()?;
    write_fixture(
        dir.path(),
        "models/two.json",
        r#"{"shapes":{"n#A":{"type":"service"},"n#B":{"type":"service"}}}"#,
    )?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing SeveralServices error")?;
    match error {
        Error::SeveralServices(file) => assert_eq!(file, dir.path().join("models/two.json")),
        other => return Err(format!("expected SeveralServices, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn invalid_json_retains_filename_and_parser_error() -> Result<(), Box<dyn std::error::Error>> {
    // Given malformed JSON, when loaded, then both path and parser evidence survive.
    let dir = tempfile::tempdir()?;
    write_fixture(dir.path(), "models/broken.json", "{")?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing InvalidJson error")?;
    match error {
        Error::InvalidJson { file, source } => {
            assert_eq!(file, dir.path().join("models/broken.json"));
            assert_eq!(source.classify(), serde_json::error::Category::Eof);
            assert_eq!((source.line(), source.column()), (1, 1));
        }
        other => return Err(format!("expected InvalidJson, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn sparse_oversized_json_is_refused_before_parsing() -> Result<(), Box<dyn std::error::Error>> {
    // Given an oversized sparse file with invalid content.
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("models"))?;
    let path = dir.path().join("models/ordinary.json");
    std::fs::File::create(&path)?.set_len(67_108_865)?;
    // When loading, then size wins over JSON errors, with an exact filename.
    let error = build_report(dir.path())
        .err()
        .ok_or("missing FileTooLarge error")?;
    match error {
        Error::FileTooLarge(file) => assert_eq!(file, path),
        other => return Err(format!("expected FileTooLarge, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn command_error_contains_every_modified_line() -> Result<(), Box<dyn std::error::Error>> {
    // Given a stored report with two modified lines.
    let dir = fixture()?;
    let changed = REPORT.replacen(
        "services 1\noperations 1\n",
        "services 9\noperations 8\n",
        1,
    );
    write_fixture(dir.path(), "catalog/aws-models.report", &changed)?;
    // When checking through the command function, then its typed error holds both.
    let error = run(dir.path(), dir.path(), ReportMode::Check)
        .err()
        .ok_or("missing mismatch")?;
    match error {
        Error::ReportMismatch(lines) => assert_eq!(
            lines,
            concat!(
                "line 1 differs: expected Some(\"services 1\\n\"), got Some(\"services 9\\n\")\n",
                "line 2 differs: expected Some(\"operations 1\\n\"), got Some(\"operations 8\\n\")\n"
            )
        ),
        other => return Err(format!("expected ReportMismatch, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn missing_models_and_empty_models_are_distinct() -> Result<(), Box<dyn std::error::Error>> {
    // Given a missing models directory, when loading, then the path accompanies I/O.
    let dir = tempfile::tempdir()?;
    let error = build_report(dir.path()).err().ok_or("missing I/O error")?;
    match error {
        Error::Io { file, source } => {
            assert_eq!(file, dir.path().join("models"));
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        other => return Err(format!("expected Io, got {other:?}").into()),
    }
    // Given an empty directory, when loading, then it is specifically NoModels.
    std::fs::create_dir(dir.path().join("models"))?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing NoModels error")?;
    assert!(matches!(error, Error::NoModels), "{error:?}");
    Ok(())
}

#[test]
fn exactly_sixty_four_mib_reaches_the_json_reader() -> Result<(), Box<dyn std::error::Error>> {
    // Given a sparse file exactly at the size limit, with invalid JSON content.
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("models"))?;
    let path = dir.path().join("models/at-limit.json");
    std::fs::File::create(&path)?.set_len(67_108_864)?;
    // When loading, then the limit is inclusive and parsing, not size, fails.
    let error = build_report(dir.path())
        .err()
        .ok_or("missing InvalidJson error")?;
    match error {
        Error::InvalidJson { file, source } => {
            assert_eq!(file, path);
            assert_eq!(source.classify(), serde_json::error::Category::Syntax);
            assert_eq!((source.line(), source.column()), (1, 1));
        }
        other => return Err(format!("expected InvalidJson at size limit, got {other:?}").into()),
    }
    Ok(())
}
