use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn args(services: &[&str]) -> Result<RunArgs, HarnessError> {
    let arguments = ["run", "--work", "unused-test-work"]
        .into_iter()
        .map(str::to_owned)
        .chain(
            services
                .iter()
                .flat_map(|service| ["--service".to_owned(), (*service).to_owned()]),
        )
        .collect::<Vec<_>>();
    RunArgs::parse(&arguments)
}

#[test]
fn given_lambda_recipe_with_signer_resources_when_selecting_collectors_then_profiles_are_collected()
-> TestResult {
    let services = collector_services(&args(&["lambda", "sts"])?)?;
    assert_eq!(services, ["lambda", "sts", "signer"]);
    Ok(())
}

#[test]
fn given_explicit_qualified_prerequisite_when_selecting_collectors_then_user_scope_is_not_broadened()
-> TestResult {
    let requested = [
        "lambda:ListCodeSigningConfigs",
        "signer:ListSigningProfiles",
    ];
    assert_eq!(collector_services(&args(&requested)?)?, requested);
    Ok(())
}

#[test]
fn given_no_cross_service_or_full_selection_when_selecting_collectors_then_selection_is_unchanged()
-> TestResult {
    assert_eq!(collector_services(&args(&["ec2", "sts"])?)?, ["ec2", "sts"]);
    assert!(collector_services(&args(&[])?)?.is_empty());
    assert!(collector_services(&args(&["sts"])?).is_err());
    Ok(())
}
