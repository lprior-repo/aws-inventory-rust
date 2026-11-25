use super::tests::store;
use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn database() -> Result<(tempfile::TempDir, String), Box<dyn std::error::Error>> {
    let work = tempfile::tempdir()?;
    let path = work.path().join("inventory.sqlite");
    let rows = [
        ("123456789012", "us-east-1", "AWS::SNS::Topic", "shared"),
        ("123456789012", "us-west-2", "AWS::SNS::Topic", "shared"),
        ("123456789012", "us-west-2", "AWS::SNS::Topic", "west-only"),
        ("123456789012", "global", "AWS::IAM::Role", "worker"),
        ("123456789012", "global", "AWS::IAM::Role", "another-global"),
        ("123456789012", "us-west-2", "AWS::IAM::Role", "worker"),
        ("123456789012", "us-west-2", "AWS::IAM::Role", "west-role"),
        ("111111111111", "global", "AWS::IAM::Role", "foreign-global"),
    ]
    .into_iter()
    .map(|(account, region, kind, id)| store::resource(account, region, kind, id))
    .collect::<Result<Vec<_>, _>>()?;
    store::sqlite(&path, &rows).await?;
    Ok((work, format!("sqlite://{}", path.display())))
}

#[tokio::test]
async fn given_regional_and_global_rows_when_reading_then_only_command_region_and_global_match()
-> TestResult {
    let (_work, url) = database().await?;
    let mut reader = Reader::connect(&url).await?;
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-east-1", None)
            .await?,
        BTreeSet::from(["shared".to_owned()])
    );
    assert_eq!(
        reader
            .identities("AWS::IAM::Role", "us-east-1", Some("123456789012"))
            .await?,
        BTreeSet::from(["worker".to_owned(), "another-global".to_owned()])
    );
    reader.close().await?;
    Ok(())
}

#[tokio::test]
async fn given_multiple_accounts_for_kind_when_account_omitted_then_error_names_sorted_accounts()
-> TestResult {
    let (work, url) = database().await?;
    store::append(
        &work.path().join("inventory.sqlite"),
        &[
            store::resource(
                "111111111111",
                "us-east-1",
                "AWS::SNS::Topic",
                "other-account",
            )?,
            store::resource(
                "111111111111",
                "us-west-2",
                "AWS::SNS::Topic",
                "other-account",
            )?,
        ],
    )
    .await?;
    let mut reader = Reader::connect(&url).await?;
    let error = reader
        .identities("AWS::SNS::Topic", "us-east-1", None)
        .await
        .err()
        .ok_or("expected account ambiguity")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control kind AWS::SNS::Topic has multiple inventory accounts: [\"111111111111\", \"123456789012\"]; specify --account"
    );
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-east-1", Some("123456789012"))
            .await?,
        BTreeSet::from(["shared".to_owned()])
    );
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-west-2", Some("111111111111"))
            .await?,
        BTreeSet::from(["other-account".to_owned()])
    );
    reader.close().await?;
    Ok(())
}

#[tokio::test]
async fn given_another_account_only_in_another_region_when_account_omitted_then_visible_account_is_used()
-> TestResult {
    let (work, url) = database().await?;
    store::append(
        &work.path().join("inventory.sqlite"),
        &[store::resource(
            "111111111111",
            "eu-west-1",
            "AWS::SNS::Topic",
            "foreign-topic",
        )?],
    )
    .await?;
    let mut reader = Reader::connect(&url).await?;
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-east-1", None)
            .await?,
        BTreeSet::from(["shared".to_owned()])
    );
    reader.close().await?;
    Ok(())
}
