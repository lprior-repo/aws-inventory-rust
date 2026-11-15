include!("smithy_helpers.rs");

use catalog_tool::ReportMode;
use catalog_tool::smithy::{Error, build_report, run};

#[test]
fn no_service_has_its_own_filename_bearing_variant() -> Result<(), Box<dyn std::error::Error>> {
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
    let dir = tempfile::tempdir()?;
    write_fixture(dir.path(), "models/broken.json", "{")?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing InvalidJson error")?;
    match error {
        Error::ModelSource(source) => {
            assert!(source.to_string().contains("models/broken.json"));
            let parser_error = std::error::Error::source(&source)
                .and_then(|source| source.downcast_ref::<serde_json::Error>())
                .ok_or("missing parser source")?;
            assert_eq!(parser_error.classify(), serde_json::error::Category::Eof);
            assert_eq!((parser_error.line(), parser_error.column()), (1, 1));
        }
        other => return Err(format!("expected ModelSource, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn sparse_oversized_json_is_refused_before_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("models"))?;
    let path = dir.path().join("models/ordinary.json");
    std::fs::File::create(&path)?.set_len(67_108_865)?;

    let error = build_report(dir.path())
        .err()
        .ok_or("missing FileTooLarge error")?;
    match error {
        Error::ModelSource(source) => assert_eq!(
            source.to_string(),
            format!("file exceeds 67108864 bytes: {}", path.display())
        ),
        other => return Err(format!("expected ModelSource, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn command_error_contains_every_modified_line() -> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let changed = REPORT.replacen(
        "services 1\noperations 1\n",
        "services 9\noperations 8\n",
        1,
    );
    write_fixture(dir.path(), "catalog/aws-models.report", &changed)?;

    let error = run(dir.path(), dir.path(), ReportMode::Check)
        .err()
        .ok_or("missing mismatch")?;
    match error {
        Error::Report(catalog_tool::ReportError::Mismatch(lines)) => assert_eq!(
            lines,
            concat!(
                "line 1 differs: expected Some(\"services 1\\n\"), got Some(\"services 9\\n\")\n",
                "line 2 differs: expected Some(\"operations 1\\n\"), got Some(\"operations 8\\n\")\n"
            )
        ),
        other => return Err(format!("expected Report(Mismatch), got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn missing_models_and_empty_models_are_distinct() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let error = build_report(dir.path()).err().ok_or("missing I/O error")?;
    match error {
        Error::ModelSource(source) => {
            assert!(source.to_string().contains("models"));
            let io_error = std::error::Error::source(&source)
                .and_then(|source| source.downcast_ref::<std::io::Error>())
                .ok_or("missing filesystem source")?;
            assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
        }
        other => return Err(format!("expected ModelSource, got {other:?}").into()),
    }

    std::fs::create_dir(dir.path().join("models"))?;
    let error = build_report(dir.path())
        .err()
        .ok_or("missing NoModels error")?;
    assert!(matches!(
        error,
        Error::ModelSource(source) if source.to_string() == "no models found"
    ));
    Ok(())
}

#[test]
fn exactly_sixty_four_mib_reaches_the_json_reader() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("models"))?;
    let path = dir.path().join("models/at-limit.json");
    std::fs::File::create(&path)?.set_len(67_108_864)?;

    let error = build_report(dir.path())
        .err()
        .ok_or("missing InvalidJson error")?;
    match error {
        Error::ModelSource(source) => {
            assert!(source.to_string().contains(&path.display().to_string()));
            let parser_error = std::error::Error::source(&source)
                .and_then(|source| source.downcast_ref::<serde_json::Error>())
                .ok_or("missing parser source")?;
            assert_eq!(parser_error.classify(), serde_json::error::Category::Syntax);
            assert_eq!((parser_error.line(), parser_error.column()), (1, 1));
        }
        other => return Err(format!("expected ModelSource at size limit, got {other:?}").into()),
    }
    Ok(())
}
