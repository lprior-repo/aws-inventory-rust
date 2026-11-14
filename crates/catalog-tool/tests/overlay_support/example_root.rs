use super::{TestResult, name, path};
use catalog_tool::overlay::*;

pub fn root_fields(o: &ServiceOverlay) -> TestResult {
    let op = o
        .operations()
        .get(&name("DescribeInstances")?)
        .ok_or("missing root")?;
    assert_eq!(
        (op.class, op.scope),
        (Some(OverlayClass::Enumerate), Some(Scope::Region))
    );
    assert_eq!(op.items, Some(path("Reservations.Instances")?));
    assert_eq!(op.page_size, Some(1000));
    assert_eq!(
        op.exclude_from_hash,
        vec![path("LaunchTime")?, path("State.Code")?]
    );
    assert_eq!(op.skip, vec![name("DescribeInstanceStatus")?]);
    assert_eq!(op.quota_group.as_deref(), Some("describe"));
    assert_eq!(op.queryable, vec![path("InstanceType")?]);
    assert_eq!(op.tags, Some(path("Tags")?));
    root_token(op)?;
    assert_eq!(op.identity, Some(Identity::Arn(path("InstanceArn")?)));
    assert_eq!(op.bind.len(), 0);
    assert_eq!(op.batch, None);
    fixed_fields(op)
}
fn root_token(op: &OperationOverlay) -> TestResult {
    match &op.paginator {
        Some(Paginator::Token(p)) => {
            assert_eq!(
                (&p.input, &p.output),
                (&path("NextToken")?, &path("NextToken")?)
            );
            Ok(())
        }
        _ => Err("wrong root paginator".into()),
    }
}
fn fixed_fields(op: &OperationOverlay) -> TestResult {
    assert_eq!(op.fixed.len(), 4);
    assert_eq!(
        op.fixed.get(&path("Filters.Name")?),
        Some(&FixedValue::String("owner-id".into()))
    );
    assert_eq!(
        op.fixed.get(&path("Limit")?),
        Some(&FixedValue::Integer(42))
    );
    assert_eq!(
        op.fixed.get(&path("Enabled")?),
        Some(&FixedValue::Boolean(true))
    );
    assert_eq!(
        op.fixed.get(&path("Owners")?),
        Some(&FixedValue::Strings(vec![
            "self".into(),
            "123456789012".into()
        ]))
    );
    Ok(())
}
