use inventory_core::inventory::Region;

#[test]
fn rust_region_cursors_follow_sqlite_binary_text_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut regions = vec![
        Region::parse("us-east-1")?,
        Region::Global,
        Region::parse("eu-west-1")?,
    ];
    regions.sort();
    let names: Vec<_> = regions.iter().map(Region::as_str).collect();
    assert_eq!(names, ["eu-west-1", "global", "us-east-1"]);
    Ok(())
}
