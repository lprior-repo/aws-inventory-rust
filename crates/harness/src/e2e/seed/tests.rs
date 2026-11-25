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
    let services = collector_services(&args(&["lambda"])?)?;
    assert_eq!(services, ["lambda", "ec2", "iam", "signer", "sqs"]);
    Ok(())
}

#[test]
fn given_qualified_recipe_selectors_when_selecting_collectors_then_user_scope_is_not_broadened()
-> TestResult {
    let requested = ["lambda:ListCodeSigningConfigs", "ec2:DescribeVpcs"];
    assert_eq!(
        collector_services(&args(&requested)?)?,
        [
            "lambda:ListCodeSigningConfigs",
            "ec2:DescribeVpcs",
            "iam",
            "signer",
            "sqs"
        ]
    );
    Ok(())
}

#[test]
fn given_no_cross_service_or_full_selection_when_selecting_collectors_then_selection_is_unchanged()
-> TestResult {
    assert_eq!(collector_services(&args(&["ec2"])?)?, ["ec2"]);
    assert!(collector_services(&args(&[])?)?.is_empty());
    assert!(collector_services(&args(&["sts"])?).is_err());
    Ok(())
}

#[test]
fn given_mixed_known_and_missing_recipe_selectors_when_selected_then_neither_is_silently_dropped()
-> TestResult {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("ec2.toml"), "")?;
    let mut args = args(&["ec2:DescribeVpcs", "nosuch:ListThings"])?;
    args.seeds = root.path().into();
    assert!(matches!(
        selected_recipes(&args),
        Err(SeedError::Invalid("requested seed service has no file"))
    ));
    assert!(matches!(
        collector_services(&args),
        Err(HarnessError::Invalid(_))
    ));
    args.services = vec!["ec2:DescribeVpcs".into(), "ec2:DescribeSubnets".into()];
    assert_eq!(selected_recipes(&args)?, ["ec2"]);
    args.services.clear();
    assert!(selected_recipes(&args)?.is_empty());
    Ok(())
}
