mod resource_servers;

use super::*;

const POOLS: [&str; 2] = ["us-east-1_PoolA", "us-east-1_PoolB"];

fn pools() -> Json {
    json!({"UserPools":POOLS.map(|pool| json!({"Id":pool,"Name":"same"}))})
}

fn case(operation: &'static str, kind: &'static str, items: &'static str) -> Case {
    let owner = |pool: &str| {
        let page = |later: bool| {
            let name = if later { "same" } else { "early" };
            let terms_id = if later {
                "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
            } else {
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
            };
            let (row, child_id) = match operation {
                "ListGroups" => (
                    json!({"UserPoolId":pool,"GroupName":name,"Description":"configuration only",
                    "RoleArn":"arn:aws:iam::123456789012:role/group-role","Precedence":1,
                    "CreationDate":1767225600,"LastModifiedDate":1767312000}),
                    name,
                ),
                "ListIdentityProviders" => (
                    json!({"ProviderName":name,"ProviderType":"SAML",
                    "CreationDate":1767225600,"LastModifiedDate":1767312000}),
                    name,
                ),
                "ListResourceServers" => (
                    json!({"UserPoolId":pool,"Identifier":name,"Name":"owned",
                    "Scopes":[{"ScopeName":"read","ScopeDescription":"read owned metadata"}]}),
                    name,
                ),
                _ => (
                    json!({"TermsId":terms_id,"TermsName":name,"Enforcement":"NONE",
                    "CreationDate":1767225600,"LastModifiedDate":1767312000}),
                    terms_id,
                ),
            };
            let request = if later {
                json!({"UserPoolId":pool,"NextToken":"later"})
            } else {
                json!({"UserPoolId":pool})
            };
            let body = if operation == "ListResourceServers" {
                let provider = json!({"UserPoolId":pool,"Identifier":"provider","Name":"Cognito managed","ManagedBy":"COGNITO"});
                let rows = if later {
                    vec![row, provider]
                } else {
                    vec![provider, row]
                };
                if later {
                    json!({items:rows})
                } else {
                    json!({items:rows,"NextToken":"later"})
                }
            } else {
                response(items, row, "NextToken", later)
            };
            Page {
                body,
                id: format!("{}:{pool}|{}:{child_id}", pool.len(), child_id.len()),
                wire: NativeRequest::post("/".to_owned(), request),
            }
        };
        Owner {
            value: pool.to_owned(),
            pages: [page(false), page(true)],
        }
    };
    Case {
        operation,
        kind,
        parent_field: "UserPoolId",
        requires_request_identity: !matches!(operation, "ListGroups" | "ListResourceServers"),
        owners: POOLS.map(owner),
    }
}

#[tokio::test]
async fn given_equal_owned_configuration_ids_and_provider_resource_servers_in_two_pools_when_later_pages_resume_then_only_owned_pool_qualified_rows_survive()
-> TestResult {
    let cases = vec![
        case("ListGroups", "AWS::Cognito::UserPoolGroup", "Groups"),
        case(
            "ListIdentityProviders",
            "AWS::Cognito::UserPoolIdentityProvider",
            "Providers",
        ),
        case(
            "ListResourceServers",
            "AWS::Cognito::UserPoolResourceServer",
            "ResourceServers",
        ),
        case("ListTerms", "AWS::Cognito::Terms", "Terms"),
    ];
    let excluded = [
        "DescribeIdentityProvider",
        "DescribeTerms",
        "ListUserPoolClientSecrets",
        "AdminGetUser",
    ];
    let operations = [
        "ListUserPools",
        "ListGroups",
        "ListIdentityProviders",
        "ListTerms",
        "ListResourceServers",
        "DescribeIdentityProvider",
        "DescribeTerms",
        "ListUserPoolClientSecrets",
        "AdminGetUser",
    ];
    run(
        "cognito-identity-provider",
        "2016-04-18",
        "ListUserPools",
        "Id",
        pools(),
        &operations,
        &excluded,
        cases,
    )
    .await
}
