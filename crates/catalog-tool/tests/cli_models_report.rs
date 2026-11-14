include!("smithy_helpers.rs");

#[test]
fn write_regenerates_and_check_prints_the_complete_report() -> Result<(), Box<dyn std::error::Error>>
{
    // Given a model and no committed report.
    let dir = fixture()?;
    // When writing, then the report is created and canonical text is printed.
    success(&report(dir.path(), &["--write"])?, REPORT);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("catalog/aws-models.report"))?,
        REPORT
    );
    // Given stale text, when writing again, then it is overwritten, not rejected.
    write_fixture(dir.path(), "catalog/aws-models.report", "stale\n")?;
    success(&report(dir.path(), &["--write"])?, REPORT);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("catalog/aws-models.report"))?,
        REPORT
    );
    success(&report(dir.path(), &["--check"])?, REPORT);
    Ok(())
}

#[test]
fn discovery_reads_all_json_recursively_and_nothing_outside_models()
-> Result<(), Box<dyn std::error::Error>> {
    // Given ordinary .json and .api.json files, plus invalid files outside models.
    let dir = fixture()?;
    write_fixture(dir.path(), "models/another/version/api.api.json", MODEL)?;
    write_fixture(dir.path(), "models/readme.txt", "invalid")?;
    write_fixture(dir.path(), "outside.json", "invalid")?;
    write_fixture(dir.path(), "other/deep/ignored.normal.json", "invalid")?;
    // When recursively reporting, then both JSON models and shared names count.
    success(&report(dir.path(), &[])?, TWO_REPORT);
    Ok(())
}

#[test]
fn check_reports_every_changed_line_and_preserves_whitespace()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a report with two changed lines and an extra line.
    let dir = fixture()?;
    let changed = REPORT.replacen(
        "services 1\noperations 1\n",
        "services 9 \noperations 1\r\n",
        1,
    );
    write_fixture(
        dir.path(),
        "catalog/aws-models.report",
        &format!("{changed}extra\n"),
    )?;
    // When checking, then the report still prints and every difference is diagnosed.
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, REPORT.as_bytes());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        concat!(
            "line 1 differs: expected Some(\"services 1\\n\"), got Some(\"services 9 \\n\")\n",
            "line 2 differs: expected Some(\"operations 1\\n\"), got Some(\"operations 1\\r\\n\")\n",
            "line 17 differs: expected None, got Some(\"extra\\n\")\n\n"
        )
    );
    Ok(())
}

#[test]
fn missing_report_still_prints_and_names_the_missing_path() -> Result<(), Box<dyn std::error::Error>>
{
    // Given models but no committed report.
    let dir = fixture()?;
    // When checking, then stdout is complete and stderr names the failed file.
    let output = report(dir.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, REPORT.as_bytes());
    let path = dir.path().join("catalog/aws-models.report");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "{}: No such file or directory (os error 2)\n",
            path.display()
        )
    );
    Ok(())
}

#[test]
fn malformed_later_model_never_prints_or_overwrites_partial_results()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a valid earlier file, a malformed later file, and an existing report.
    let dir = fixture()?;
    write_fixture(dir.path(), "models/z-last.json", "{")?;
    write_fixture(dir.path(), "catalog/aws-models.report", "preserve\n")?;
    // When writing, then loading fails before stdout or the report changes.
    let output = report(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("catalog/aws-models.report"))?,
        "preserve\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "invalid JSON in {}: EOF while parsing an object at line 1 column 1\n",
            dir.path().join("models/z-last.json").display()
        )
    );
    Ok(())
}

#[test]
fn defaults_source_to_root_local_checkout() -> Result<(), Box<dyn std::error::Error>> {
    // Given models only at the root-relative default source.
    let dir = tempfile::tempdir()?;
    write_fixture(dir.path(), ".local/aws-models/models/service.json", MODEL)?;
    // When source is omitted, then that checkout is used.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "report", "--root"])
        .arg(dir.path())
        .output()?;
    success(&output, REPORT);
    Ok(())
}

#[test]
fn rejects_missing_unknown_duplicate_and_conflicting_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    // Given malformed command lines, when invoked, then syntax fails before I/O.
    let cases: &[(&[&str], &str)] = &[
        (&["models"], "expected 'report' subcommand"),
        (&["models", "list"], "unexpected 'list'"),
        (&["models", "report", "--help"], "unexpected '--help'"),
        (
            &["models", "report", "--source"],
            "--source requires a value",
        ),
        (&["models", "report", "--root"], "--root requires a value"),
        (
            &["models", "report", "--source", "--write"],
            "--source requires a value",
        ),
        (
            &["models", "report", "--source", "x", "--source", "y"],
            "repeated or conflicting flag --source",
        ),
        (
            &["models", "report", "--root", "x", "--root", "y"],
            "repeated or conflicting flag --root",
        ),
        (
            &["models", "report", "--write", "--write"],
            "repeated or conflicting flag --write",
        ),
        (
            &["models", "report", "--check", "--check"],
            "repeated or conflicting flag --check",
        ),
        (
            &["models", "report", "--check", "--write"],
            "repeated or conflicting flag --write",
        ),
        (
            &["models", "report", "--write", "--check"],
            "repeated or conflicting flag --check",
        ),
    ];
    cases.iter().try_for_each(|(args, message)| {
        let output = tool(args)?;
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert_eq!(output.stdout, b"");
        assert!(String::from_utf8(output.stderr)?.starts_with(&format!("{message}; usage:\n")));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
