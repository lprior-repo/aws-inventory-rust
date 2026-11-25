use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const A: &str = "AWS::SNS::Topic\tmissing_from_sweep:a*\tfirst\n";
const Z: &str = "AWS::SNS::Topic\tmissing_from_sweep:z*\tlast\n";

fn args(work: &Path) -> Result<Args, HarnessError> {
    let args = Args {
        database: "postgres://localhost/inventory".into(),
        endpoint: "https://localhost".into(),
        ca_bundle: work.join("ca.pem"),
        region: "us-east-1".into(),
        account: None,
        report: work.join("report.tsv"),
        catalog: work.join("catalog"),
        universal: work.join("universal"),
        differences: work.join("differences.tsv"),
        seeds: work.join("seeds"),
        oracle: None,
    };
    [&args.catalog, &args.universal, &args.ca_bundle]
        .into_iter()
        .try_for_each(|path| std::fs::write(path, "input"))?;
    Ok(args)
}

fn report() -> Report {
    Report {
        text: "report\n".into(),
        unexplained_seeded: 0,
        not_comparable_seeded: 0,
    }
}

#[tokio::test]
async fn given_human_rules_when_report_is_written_then_original_bytes_and_timestamp_are_preserved()
-> TestResult {
    let work = tempfile::tempdir()?;
    let args = args(work.path())?;
    let text = format!("# handwritten\r\n\r\n{Z}{A}{A}");
    std::fs::write(&args.differences, &text)?;
    let past = std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(946_684_800))
        .ok_or("invalid fixed timestamp")?;
    std::fs::File::open(&args.differences)?.set_modified(past)?;
    write(&args, report()).await?;
    assert_eq!(std::fs::read_to_string(&args.differences)?, text);
    assert_eq!(std::fs::metadata(&args.differences)?.modified()?, past);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn given_report_directory_when_publishing_then_io_error_kind_is_preserved() -> TestResult {
    let work = tempfile::tempdir()?;
    let args = args(work.path())?;
    std::fs::create_dir(&args.report)?;
    let error = write(&args, report())
        .await
        .err()
        .ok_or("expected report error")?;
    let HarnessError::Io(error) = error else {
        return Err("expected IO error".into());
    };
    assert_eq!(error.kind(), std::io::ErrorKind::IsADirectory);
    assert!(!args.differences.try_exists()?);
    assert!(args.report.is_dir());
    Ok(())
}

#[test]
fn given_output_in_seed_directory_when_validated_then_it_is_refused() -> TestResult {
    let work = tempfile::tempdir()?;
    let mut args = args(work.path())?;
    args.report = args.seeds.clone();
    let error = validate(&args).err().ok_or("expected seeds collision")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control outputs must not replace the seeds directory or its contents"
    );
    std::fs::create_dir(&args.seeds)?;
    let paths = [args.seeds.clone(), args.seeds.join("report.tsv")];
    paths.into_iter().try_for_each(|path| -> TestResult {
        args.report = path;
        let error = validate(&args).err().ok_or("expected seeds collision")?;
        assert_eq!(
            error.to_string(),
            "Cloud Control outputs must not replace the seeds directory or its contents"
        );
        Ok(())
    })?;
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn given_report_when_created_then_mode_follows_normal_file_creation() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let work = tempfile::tempdir()?;
    let args = args(work.path())?;
    let control = work.path().join("control");
    std::fs::write(&control, "control")?;
    write(&args, report()).await?;
    let expected = std::fs::metadata(control)?.permissions().mode() & 0o777;
    assert_eq!(
        std::fs::metadata(&args.report)?.permissions().mode() & 0o777,
        expected
    );
    Ok(())
}
