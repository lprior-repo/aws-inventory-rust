use super::trust_and_metadata::listed_role;
use super::*;

#[test]
fn given_invalid_recognized_principal_arns_when_list_roles_produces_trust_references_then_the_page_fails_closed()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    for (principal_type, resource) in [
        ("AWS", "role/target#"),
        ("AWS", "user/bad%name"),
        ("Federated", "saml-provider/path/name"),
        ("Federated", "oidc-provider/example.com:443"),
    ] {
        let arn = format!("arn:aws:iam::210987654321:{resource}");
        let principal = std::collections::BTreeMap::from([(principal_type, arn.as_str())]);
        let document = serde_json::to_string(&json!({"Statement":[{
            "Effect":"Allow", "Action":"sts:AssumeRole", "Principal":principal
        }]}))?;
        let observed = observations(plan, rows(plan, &listed_role(&document))?)?;
        let observation = observed.first().ok_or("projected role absent")?;
        assert!(
            matches!(
                crate::resource_projection::trust_references(&catalog, plan, observation),
                Err(SweepError::Operation("invalid reviewed IAM principal ARN"))
            ),
            "accepted invalid principal {arn}"
        );
    }
    Ok(())
}

#[test]
fn given_punctuated_iam_paths_when_list_roles_produces_trust_references_then_concrete_role_targets_are_preserved_exactly()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    for path in [
        "team:prod",
        "team#prod",
        "team%prod",
        r"team\prod",
        r"team:#%\prod",
    ] {
        let arn = format!("arn:aws:iam::210987654321:role/{path}/external_+=,.@-");
        let document = serde_json::to_string(&json!({"Statement":[{
            "Effect":"Allow", "Action":"sts:AssumeRole", "Principal":{"AWS":arn}
        }]}))?;
        let observed = observations(plan, rows(plan, &listed_role(&document))?)?;
        let observation = observed.first().ok_or("projected role absent")?;
        let references = crate::resource_projection::trust_references(&catalog, plan, observation)?;
        assert_eq!(references.len(), 1, "lost valid path principal {arn}");
        let reference = references
            .first()
            .ok_or("valid path principal reference absent")?;
        assert_eq!(reference.relation.as_str(), "DECLARES_TRUST");
        assert_eq!(reference.target.kind().as_str(), "AWS::IAM::Role");
        assert_eq!(reference.target.account().as_str(), "210987654321");
        assert_eq!(reference.target.region(), &Region::Global);
        assert_eq!(reference.target.value().as_str(), arn);
        assert_eq!(reference.from.account.as_str(), "123456789012");
    }
    Ok(())
}
