use super::*;
use crate::overlay::MemberPath;

#[test]
fn given_lowercase_arn_when_generated_then_exact_model_casing_is_preserved() -> Result {
    for field in ["arn", "thingArn"] {
        let mut value = given_model();
        set(
            &mut value,
            "/shapes/n#Thing",
            "members",
            json!({field:{"target":"smithy.api#String"}}),
        )?;
        let entry = then_callable(derived(
            &value,
            &Vocabulary::default(),
            &DenyList::builtin(),
        )?)?;
        assert_eq!(entry.identity, Some(Identity::Arn(MemberPath::new(field)?)));
    }
    Ok(())
}

#[test]
fn given_ambiguous_case_variants_when_generated_then_identity_is_not_guessed() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Thing/members",
        "arn",
        json!({"target":"smithy.api#String"}),
    )?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "identity: no ARN member and no modeled identifiers",
    )
}

#[test]
fn given_scalar_map_values_when_generated_then_values_are_not_resource_keys() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes",
        "n#Items",
        json!({"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}),
    )?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "identity: no ARN member and no modeled identifiers",
    )
}

#[test]
fn given_unannotated_secret_members_when_generated_then_secret_names_stay_noncallable() -> Result {
    for field in [
        "Password",
        "SecretAccessKey",
        "ApiKey",
        "Token",
        "Environment",
    ] {
        let mut value = given_model();
        set(
            &mut value,
            "/shapes/n#Thing/members",
            field,
            json!({"target":"smithy.api#String"}),
        )?;
        then_deferred(
            derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
            &format!("output: secret-bearing member {field}"),
        )?;
    }
    Ok(())
}

#[test]
fn given_secret_arn_reference_when_generated_then_reference_is_not_treated_as_secret_value()
-> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Thing/members",
        "SecretArn",
        json!({"target":"smithy.api#String"}),
    )?;
    then_callable(derived(
        &value,
        &Vocabulary::default(),
        &DenyList::builtin(),
    )?)?;
    Ok(())
}
