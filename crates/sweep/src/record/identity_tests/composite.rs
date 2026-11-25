use super::*;
#[test]
fn compiled_composite_identity_accepts_exact_bound_and_rejects_oversize_components() -> TestResult {
    let bytes = compiled(
        Identity::Key(vec![
            MemberPath::new("Name")?,
            MemberPath::new("OtherName")?,
        ]),
        "active-wire",
        false,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("things", "ListThings")
        .ok_or("plan absent")?;
    let encode = |first: &str, second: &str| -> TestResult<Record> {
        Ok(Record::encode(&Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(6)?), Value::Text(first.to_owned())),
            (Key::Field(FieldId::new(7)?), Value::Text(second.to_owned())),
        ])))?)
    };
    let first = "a".repeat(2042);
    let second = "b".repeat(2043);
    let accepted = resource_id(&encode(&first, &second)?, plan.identity(), None)?;
    assert_eq!(accepted.as_str(), format!("2042:{first}|2043:{second}"));
    assert_eq!(accepted.as_str().len(), 4096);
    [(2043, 2043), (1_048_576, 1), (1, 1_048_576)]
        .into_iter()
        .try_for_each(|(first, second)| -> TestResult {
            let result = resource_id(
                &encode(&"a".repeat(first), &"b".repeat(second))?,
                plan.identity(),
                None,
            );
            assert!(matches!(
                result,
                Err(SweepError::Operation(
                    "modeled resource identity is too long"
                ))
            ));
            Ok(())
        })
}
