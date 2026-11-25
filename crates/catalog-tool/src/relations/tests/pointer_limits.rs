use super::*;
use serde_json::json;

fn catalog(segments: &[&str], lists: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut shapes = serde_json::Map::from_iter([
        (
            "n#Service".to_owned(),
            json!({"type":"service","version":"1","operations":[{"target":"n#ListItems"}],
            "traits":{"aws.api#service":{"sdkId":"Pointer","endpointPrefix":"pointer"}}}),
        ),
        (
            "n#ListItems".to_owned(),
            json!({"type":"operation","output":{"target":"n#Node0"}}),
        ),
    ]);
    for (index, name) in segments.iter().enumerate() {
        let next = index.checked_add(1).ok_or("depth overflow")?;
        let target = if next == segments.len() {
            "smithy.api#String".to_owned()
        } else {
            format!("n#Node{next}")
        };
        let member_target = if lists {
            shapes.insert(
                format!("n#List{index}"),
                json!({"type":"list","member":{"target":target}}),
            );
            format!("n#List{index}")
        } else {
            target
        };
        shapes.insert(
            format!("n#Node{index}"),
            json!({"type":"structure","members":{
                (*name):{"target":member_target}
            }}),
        );
    }
    let source = tempfile::tempdir()?;
    let directory = source.path().join("models/pointer/service/2020-01-01");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("model.json"),
        serde_json::to_vec(&json!({"smithy":"2.0","shapes":shapes}))?,
    )?;
    let fields = segments
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let registry = smithy::Registry::parse(&format!("{fields}\n"))?;
    Ok(smithy::build_catalog(
        source.path(),
        &registry,
        "pointer-limits",
    )?)
}

fn resolve(bytes: &[u8], pointer: &str) -> Result<pointer::ResolvedPointer, String> {
    let catalog = Catalog::borrow(bytes).map_err(|error| error.to_string())?;
    let service = ServiceIndex::build(catalog)
        .and_then(|services| services.get("pointer"))
        .map_err(|error| error.to_string())?;
    let item = model::output(service, "ListItems").map_err(|error| error.to_string())?;
    pointer::resolve(service, item, pointer).map_err(|error| error.to_string())
}

#[test]
fn given_1024_byte_pointer_when_resolved_then_member_lookup_runs_but_1025_bytes_is_invalid()
-> TestResult {
    let bytes = catalog(&["Id"], false)?;
    let segment = "X".repeat(1023);
    let at_limit = format!("/{segment}");
    assert_eq!(at_limit.len(), 1024);
    let error = resolve(&bytes, &at_limit)
        .err()
        .ok_or("expected missing member")?;
    assert_eq!(
        error,
        format!("pointer segment {segment} not in item shape")
    );
    let error = resolve(&bytes, &format!("{at_limit}X"))
        .err()
        .ok_or("expected invalid pointer")?;
    assert_eq!(error, "invalid pointer");
    Ok(())
}

#[test]
fn given_16_segments_when_resolved_then_list_path_is_valid_but_17_segments_is_invalid() -> TestResult
{
    let bytes = catalog(&["A"; 8], true)?;
    let pointer = "/A/*".repeat(8);
    let resolved = resolve(&bytes, &pointer)?;
    assert_eq!(resolved.path, "A.A.A.A.A.A.A.A");
    assert_eq!(resolved.traversal, pointer::Traversal::List);
    let error = resolve(&bytes, &format!("{pointer}/A"))
        .err()
        .ok_or("expected invalid pointer")?;
    assert_eq!(error, "invalid pointer");
    Ok(())
}

#[test]
fn given_256_byte_member_path_when_resolved_then_it_is_valid_but_257_bytes_is_invalid() -> TestResult
{
    let member = "A".repeat(256);
    let bytes = catalog(&[&member], false)?;
    let resolved = resolve(&bytes, &format!("/{member}"))?;
    assert_eq!(resolved.path, member);
    assert_eq!(resolved.traversal, pointer::Traversal::Scalar);
    let oversized = "A".repeat(257);
    let bytes = catalog(&[&oversized], false)?;
    let error = resolve(&bytes, &format!("/{oversized}"))
        .err()
        .ok_or("expected invalid path")?;
    assert_eq!(error, "invalid pointer");
    Ok(())
}

#[test]
fn given_eight_member_segments_when_resolved_then_path_is_valid_but_nine_is_invalid() -> TestResult
{
    let bytes = catalog(&["A"; 8], false)?;
    let resolved = resolve(&bytes, &"/A".repeat(8))?;
    assert_eq!(resolved.path, "A.A.A.A.A.A.A.A");
    assert_eq!(resolved.traversal, pointer::Traversal::Scalar);
    let bytes = catalog(&["A"; 9], false)?;
    let error = resolve(&bytes, &"/A".repeat(9))
        .err()
        .ok_or("expected invalid path")?;
    assert_eq!(error, "invalid pointer");
    Ok(())
}
