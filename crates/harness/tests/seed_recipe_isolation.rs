type TestResult = Result<(), Box<dyn std::error::Error>>;

fn input<'a>(
    recipe: &'a toml::Value,
    operation: &str,
    field: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let steps = recipe
        .get("step")
        .and_then(toml::Value::as_array)
        .ok_or("recipe has no steps")?;
    steps
        .iter()
        .find(|step| step.get("operation").and_then(toml::Value::as_str) == Some(operation))
        .and_then(|step| step.get("input"))
        .and_then(|values| values.get(field))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("missing {operation} input {field}").into())
}

#[test]
fn given_eks_and_emr_recipes_when_combined_then_prerequisite_identities_do_not_collide()
-> TestResult {
    let eks: toml::Value = toml::from_str(include_str!("../seeds/eks.toml"))?;
    let emr: toml::Value = toml::from_str(include_str!("../seeds/emr-containers.toml"))?;
    [
        ("CreateRole", "RoleName"),
        ("CreateCluster", "name"),
        ("CreateCluster", "clientRequestToken"),
    ]
    .into_iter()
    .try_for_each(|(operation, field)| -> TestResult {
        assert_ne!(
            input(&eks, operation, field)?,
            input(&emr, operation, field)?
        );
        Ok(())
    })?;
    assert_eq!(
        input(&emr, "CreateRole", "RoleName")?,
        input(&emr, "AttachRolePolicy", "RoleName")?
    );
    Ok(())
}
