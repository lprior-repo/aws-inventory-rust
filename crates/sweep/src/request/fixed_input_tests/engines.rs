use super::*;
#[test]
fn given_mixed_database_engines_when_query_filters_signed_then_each_request_selects_only_its_own_engine()
-> TestResult {
    let rows = [("n", "neptune"), ("d", "docdb"), ("r", "aurora-mysql")];
    ["neptune", "docdb", "aurora-mysql"]
        .into_iter()
        .zip(["n", "d", "r"])
        .try_for_each(|(engine, expected)| {
            let request = signed("aws.protocols#awsQuery", engine)?;
            let query = std::str::from_utf8(request.body())?;
            assert!(query.contains("Filters.Filter.1.Name=engine"));
            let selected = query
                .split('&')
                .find_map(|pair| pair.strip_prefix("Filters.Filter.1.Values.Value.1="))
                .ok_or("engine value absent")?;
            let observed = rows
                .iter()
                .filter(|(_, row_engine)| *row_engine == selected)
                .map(|(id, _)| *id)
                .collect::<Vec<_>>();
            assert_eq!(observed, vec![expected]);
            let other = signed(
                "aws.protocols#awsQuery",
                if engine == "neptune" {
                    "docdb"
                } else {
                    "neptune"
                },
            )?;
            assert_ne!(
                request.headers().get("authorization"),
                other.headers().get("authorization")
            );
            Ok(())
        })
}

#[test]
fn given_invalid_nested_literals_when_compiled_then_model_and_resource_bounds_reject() -> TestResult
{
    let invalid = [
        (
            overlay::FixedValue::list([overlay::FixedValue::structure([(
                "Name",
                overlay::FixedValue::String("engine".into()),
            )])?]),
            false,
        ),
        (
            with_field("Unknown", overlay::FixedValue::Boolean(true))?,
            false,
        ),
        (
            overlay::FixedValue::list([overlay::FixedValue::String("not a structure".into())]),
            false,
        ),
        (
            overlay::FixedValue::list(vec![
                overlay::FixedValue::structure([
                    ("Name", overlay::FixedValue::String("engine".into())),
                    (
                        "Values",
                        overlay::FixedValue::Strings(vec!["neptune".into()])
                    )
                ])?;
                65
            ]),
            true,
        ),
        (
            overlay::FixedValue::list([overlay::FixedValue::structure([
                ("Name", overlay::FixedValue::String("x".repeat(16_385))),
                (
                    "Values",
                    overlay::FixedValue::Strings(vec!["neptune".into()]),
                ),
            ])?]),
            true,
        ),
        (filters("invalid-engine")?, false),
        (
            with_field("Threshold", overlay::FixedValue::Integer(8))?,
            false,
        ),
    ];
    invalid
        .into_iter()
        .try_for_each(|(literal, exceeds_bounds)| {
            let error = compiled("aws.protocols#awsQuery", literal)
                .err()
                .ok_or("invalid literal accepted")?;
            if exceeds_bounds {
                assert_eq!(
                    error
                        .downcast_ref::<overlay::NameError>()
                        .ok_or("expected literal-bounds error")?
                        .value,
                    "fixed: literal bounds exceeded"
                );
            } else {
                assert_invalid_value(error.as_ref())?;
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        })?;
    Ok(())
}
