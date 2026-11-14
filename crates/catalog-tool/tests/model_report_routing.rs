include!("smithy_helpers.rs");

#[test]
fn explicit_source_and_root_route_reads_and_writes_independently()
-> Result<(), Box<dyn std::error::Error>> {
    let source = fixture()?;
    let root = tempfile::tempdir()?;

    ["--write", "--check"].into_iter().try_for_each(|flag| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
            .args(["models", "report", "--source"])
            .arg(source.path())
            .arg("--root")
            .arg(root.path())
            .arg(flag)
            .output()?;

        success(&output, REPORT);
        assert_eq!(
            std::fs::read_to_string(root.path().join("catalog/aws-models.report"))?,
            REPORT
        );
        let absent = std::fs::metadata(source.path().join("catalog/aws-models.report"))
            .err()
            .ok_or("report was incorrectly written inside source")?;
        assert_eq!(absent.kind(), std::io::ErrorKind::NotFound);
        let absent = std::fs::metadata(root.path().join("models"))
            .err()
            .ok_or("models unexpectedly appeared inside output root")?;
        assert_eq!(absent.kind(), std::io::ErrorKind::NotFound);
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
