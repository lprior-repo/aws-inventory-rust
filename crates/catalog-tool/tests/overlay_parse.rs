use catalog_tool::{classes::DenyReason, overlay::*};
pub mod overlay_support;
use overlay_support::example_root::root_fields;
use overlay_support::{EXAMPLE, TestResult, name, path};

#[test]
fn documented_example_has_exact_typed_fields() -> TestResult {
    // Given the embedded reference fixture, when parsed, then every field is exact.
    let overlay = ServiceOverlay::parse("ec2", EXAMPLE)?;
    assert_eq!(
        (
            overlay.service(),
            overlay.format(),
            overlay.reviewed(),
            overlay.reviewer()
        ),
        ("ec2", 1, "2026-11-13", "Example reviewer")
    );
    assert_eq!(overlay.operations().len(), 5);
    root_fields(&overlay)?;
    expansion_fields(&overlay)?;
    batch_fields(&overlay)?;
    denial_fields(&overlay)?;
    Ok(())
}
fn expansion_fields(o: &ServiceOverlay) -> TestResult {
    let op = o
        .operations()
        .get(&name("DescribeInstanceAttribute")?)
        .ok_or("missing expand")?;
    assert_eq!(
        (op.class, op.scope),
        (Some(OverlayClass::Expand), Some(Scope::Account))
    );
    common_absent(op);
    assert_eq!(op.fixed.len(), 0);
    assert_eq!(op.batch, None);
    assert_eq!(op.paginator, Some(Paginator::None));
    assert_eq!(
        op.identity,
        Some(Identity::Template(
            "arn:{partition}:ec2:{region}:{account}:instance/{InstanceId}".into()
        ))
    );
    assert_eq!(op.bind.len(), 3);
    assert_eq!(
        op.bind.get(&path("InstanceId")?),
        Some(&Binding::From {
            operation: name("DescribeInstances")?,
            path: path("InstanceId")?
        })
    );
    assert_eq!(op.bind.get(&path("AccountId")?), Some(&Binding::Account));
    assert_eq!(op.bind.get(&path("Region")?), Some(&Binding::Region));
    Ok(())
}
fn batch_fields(o: &ServiceOverlay) -> TestResult {
    let op = o
        .operations()
        .get(&name("BatchGetThings")?)
        .ok_or("missing batch")?;
    assert_eq!((op.class, op.scope), (None, Some(Scope::HomeRegion)));
    common_absent(op);
    assert_eq!((op.fixed.len(), op.bind.len()), (0, 0));
    let batch = op.batch.as_ref().ok_or("missing batch plan")?;
    assert_eq!(
        (&batch.member, batch.max, &batch.replaces),
        (&path("ThingIds")?, 100, &name("DescribeThing")?)
    );
    assert_eq!(op.identity, Some(Identity::Key(vec![path("ThingId")?])));
    match &op.paginator {
        Some(Paginator::Tokens { pairs, truncated }) => {
            assert_eq!(pairs.len(), 1);
            let pair = pairs.first().ok_or("missing pair")?;
            assert_eq!(
                (&pair.input, &pair.output),
                (&path("StartRecordName")?, &path("NextRecordName")?)
            );
            assert_eq!(*truncated, Some(path("IsTruncated")?));
        }
        _ => return Err("wrong paginator".into()),
    }
    Ok(())
}
fn denial_fields(o: &ServiceOverlay) -> TestResult {
    [
        ("GetObject", OverlayClass::Deny(DenyReason::Content)),
        ("GetOther", OverlayClass::Defer),
    ]
    .into_iter()
    .try_for_each(|(n, class)| -> TestResult {
        let op = o
            .operations()
            .get(&name(n)?)
            .ok_or("missing non-callable")?;
        assert_eq!((op.class, op.scope), (Some(class), None));
        common_absent(op);
        assert_eq!((op.fixed.len(), op.bind.len()), (0, 0));
        assert_eq!(
            (&op.identity, &op.paginator, &op.batch),
            (&None, &None, &None)
        );
        Ok(())
    })
}
fn common_absent(op: &OperationOverlay) {
    assert_eq!(
        (&op.items, op.page_size, &op.quota_group, &op.tags),
        (&None, None, &None, &None)
    );
    assert_eq!(
        (&op.exclude_from_hash, &op.skip, &op.queryable),
        (&vec![], &vec![], &vec![])
    );
}

#[test]
fn absent_overrides_and_deny_reason_taxonomy_are_preserved() -> TestResult {
    // Given no overrides or each denial spelling, when parsed, then defaults stay absent.
    let o = overlay_support::parse("[operation.ListThings]")?;
    let op = o
        .operations()
        .get(&name("ListThings")?)
        .ok_or("missing op")?;
    common_absent(op);
    assert_eq!((op.class, op.scope), (None, None));
    assert_eq!((op.fixed.len(), op.bind.len()), (0, 0));
    assert_eq!(
        (&op.identity, &op.paginator, &op.batch),
        (&None, &None, &None)
    );
    [
        ("content", DenyReason::Content),
        ("secret", DenyReason::Secret),
        ("paid", DenyReason::Paid),
        ("mutating", DenyReason::Mutating),
        ("reference-data", DenyReason::ReferenceData),
    ]
    .into_iter()
    .try_for_each(|(s, reason)| -> TestResult {
        let o = overlay_support::parse(&format!(
            "[operation.ListThings]\nclass='deny'\ndeny_reason='{s}'"
        ))?;
        assert_eq!(
            o.operations().get(&name("ListThings")?).map(|op| op.class),
            Some(Some(OverlayClass::Deny(reason)))
        );
        Ok(())
    })
}
