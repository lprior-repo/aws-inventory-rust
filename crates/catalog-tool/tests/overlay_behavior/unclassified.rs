use super::*;

#[test]
fn unclassified_candidate_requires_explicit_disposition() -> Result<(), Box<dyn std::error::Error>>
{
    let model = BASIC
        .replace(
            "\"n#ListThings\"}]",
            "\"n#ListThings\"},{\"target\":\"n#GetThing\"}]",
        )
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\"",
            "\"n#GetThing\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#ListThings\":{\"type\":\"operation\"",
        );
    let root = fixture(&model, "")?;
    overlay(
        root.path(),
        &format!("{HEADER}{COMPLETE}[operation.GetThing]\n"),
    )?;
    let errors = violations(root.path())?;
    assert!(errors.iter().any(|error| error.operation() == "GetThing"
        && error.key() == "review"
        && error.value() == "non-callable candidate"
        && error.rule() == ViolationRule::IncompleteReview));
    Ok(())
}

#[test]
fn an_unclassified_operation_can_be_reviewed_as_expand() -> Result<(), Box<dyn std::error::Error>> {
    let model = BASIC
        .replace(
            "\"n#ListThings\"}]",
            "\"n#ListThings\"},{\"target\":\"n#GetThing\"},{\"target\":\"n#ListDenied\"}]",
        )
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\"",
            "\"n#GetThing\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#ListDenied\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#ListThings\":{\"type\":\"operation\"",
        );
    let text = format!(
        "{HEADER}[operation.ListDenied]\nclass='deny'\ndeny_reason='content'\n[operation.ListThings]\nclass='defer'\n{}",
        COMPLETE
            .replace("ListThings", "GetThing")
            .replace("enumerate", "expand")
    );
    let root = fixture(&model, "")?;
    overlay(root.path(), &text)?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("missing callable")?;
    assert_eq!(callable.operation(), "GetThing");
    assert_eq!(callable.class(), EffectiveClass::Expand);
    assert_eq!(
        checked.report(),
        "totals\tservices_with_overlay=1\tcallable_enumerate=0\tcallable_expand=1\tdenied_by_overlay=1\tdeferred=1\tservices_without_overlay=0\tunreviewed_enumerate=0\tunreviewed_expand=0\tunreviewed_unclassified=0\noperation\tthings\tGetThing\texpand\tclass=overlay,items=overlay,page_size=missing,paginator=overlay,identity=overlay,bindings=missing,scope=missing,batch=missing\noperation\tthings\tListDenied\tdenied\tclass=overlay,items=missing,page_size=missing,paginator=missing,identity=missing,bindings=missing,scope=missing,batch=missing\noperation\tthings\tListThings\tdeferred\tclass=overlay,items=missing,page_size=missing,paginator=missing,identity=missing,bindings=missing,scope=missing,batch=missing\n"
    );
    Ok(())
}
