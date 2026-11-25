use super::*;
mod global;

struct PolicyCase {
    service: &'static str,
    parent: &'static str,
    child: &'static str,
    kind: &'static str,
    parent_kind: &'static str,
    relation: &'static str,
}

const CASES: &[PolicyCase] = &[
    PolicyCase {
        service: "iam",
        parent: "ListGroups",
        child: "ListGroupPolicies",
        kind: "AWS::IAM::GroupPolicy",
        parent_kind: "AWS::IAM::Group",
        relation: "POLICY_OF",
    },
    PolicyCase {
        service: "iam",
        parent: "ListRoles",
        child: "ListRolePolicies",
        kind: "AWS::IAM::RolePolicy",
        parent_kind: "AWS::IAM::Role",
        relation: "POLICY_OF",
    },
    PolicyCase {
        service: "iam",
        parent: "ListUsers",
        child: "ListUserPolicies",
        kind: "AWS::IAM::UserPolicy",
        parent_kind: "AWS::IAM::User",
        relation: "POLICY_OF",
    },
    PolicyCase {
        service: "iam",
        parent: "ListPolicies",
        child: "ListPolicyVersions",
        kind: "AWS::ApiOnly::IAMPolicyVersion",
        parent_kind: "AWS::IAM::ManagedPolicy",
        relation: "VERSION_OF",
    },
    PolicyCase {
        service: "kms",
        parent: "ListKeys",
        child: "ListKeyPolicies",
        kind: "AWS::ApiOnly::KMSKeyPolicy",
        parent_kind: "AWS::KMS::Key",
        relation: "POLICY_OF",
    },
];

#[tokio::test]
async fn given_shared_policy_names_under_two_owners_when_native_wire_pages_resume_then_all_children_keep_real_owner_edges()
-> TestResult {
    let catalog = fixture::catalog()?;
    for case in CASES {
        let parent = fixture::plan(&catalog, case.service, case.parent)?;
        let child = fixture::plan(&catalog, case.service, case.child)?;
        let mut scenario = Scenario::new(&catalog, case.child).await?;
        scenario.seed(parent).await?;
        let lease = scenario.claim(case.parent).await?;
        let (request, wire) = scenario.request(parent, &lease)?;
        if case.parent == "ListPolicies" {
            request_contains(&wire, "Scope=Local")?;
        }
        scenario
            .apply(&catalog, parent, lease, &request, &parents(case)?)
            .await?;
        let mut parents_seen = std::collections::BTreeSet::new();
        let mut resumptions = 0;
        for _ in 0..4 {
            let lease = scenario.claim(case.child).await?;
            let resumed = lease.token().is_some();
            let (request, wire) = scenario.request(child, &lease)?;
            let binding = child.bindings().first().ok_or("owner binding absent")?;
            let ValueRef::Text(owner) =
                path_value(request.root(), &binding.path)?.ok_or("actual owner absent")?
            else {
                return Err("actual owner is not text".into());
            };
            parents_seen.insert(owner.to_owned());
            request_contains(
                &wire,
                if owner.ends_with("parent-a") {
                    "parent-a"
                } else {
                    "parent-b"
                },
            )?;
            if case.service == "kms" {
                assert!(owner.starts_with("arn:aws:kms:us-east-1:123456789012:key/"));
            }
            if resumed {
                resumptions += 1;
                request_contains(&wire, "resume2")?;
            }
            scenario
                .apply(
                    &catalog,
                    child,
                    lease,
                    &request,
                    &policy_page(case, resumed),
                )
                .await?;
            scenario = scenario.reopen().await?;
        }
        assert_eq!(parents_seen.len(), 2);
        assert_eq!(resumptions, 2, "empty first page was silently terminal");
        let nodes = scenario.assert_nodes(case.kind, 2).await?;
        assert_ne!(
            nodes.first().ok_or("first policy absent")?.2,
            nodes.last().ok_or("second policy absent")?.2
        );
        let reader = scenario.reader().await?;
        let edges: Vec<(String, String)> = sqlx::query_as(
            "SELECT to_kind,to_id FROM edges WHERE from_kind=? AND relation=? ORDER BY to_id",
        )
        .bind(case.kind)
        .bind(case.relation)
        .fetch_all(&reader)
        .await?;
        assert_eq!(edges.len(), 2, "owner edges were not resolved");
        assert!(edges.iter().all(|(kind, _)| kind == case.parent_kind));
        assert!(
            edges
                .iter()
                .zip(["parent-a", "parent-b"])
                .all(|((_, id), name)| id.ends_with(name))
        );
        reader.close().await;
        assert!(
            scenario
                .store
                .claim(&scenario.sweep, 1000, 10000)
                .await?
                .is_none(),
            "unexpected policy-body or credential work"
        );
    }
    Ok(())
}

fn parents(case: &PolicyCase) -> TestResult<Vec<u8>> {
    if case.service == "kms" {
        return Ok(serde_json::to_vec(&json!({"Keys":[
            {"KeyId":"parent-a","KeyArn":"arn:aws:kms:us-east-1:123456789012:key/parent-a"},
            {"KeyId":"parent-b","KeyArn":"arn:aws:kms:us-east-1:123456789012:key/parent-b"}
        ],"Truncated":false}))?);
    }
    let (list, member, arn_part) = match case.parent {
        "ListGroups" => ("Groups", "Group", "group"),
        "ListRoles" => ("Roles", "Role", "role"),
        "ListUsers" => ("Users", "User", "user"),
        "ListPolicies" => ("Policies", "Policy", "policy"),
        _ => return Err("unexpected policy parent fixture".into()),
    };
    let mut records = String::new();
    for name in ["parent-a", "parent-b"] {
        let trust = if member == "Role" {
            "<AssumeRolePolicyDocument>%7B%22Statement%22%3A%5B%5D%7D</AssumeRolePolicyDocument>"
        } else {
            ""
        };
        records.push_str(&format!("<member><Path>/</Path><{member}Name>{name}</{member}Name><{member}Id>ID{name}</{member}Id><Arn>arn:aws:iam::123456789012:{arn_part}/{name}</Arn><CreateDate>2026-01-01T00:00:00Z</CreateDate>{trust}</member>"));
    }
    Ok(xml(
        case.parent,
        &format!("<{list}>{records}</{list}><IsTruncated>false</IsTruncated>"),
    ))
}

fn policy_page(case: &PolicyCase, resumed: bool) -> Vec<u8> {
    if case.service == "kms" {
        return if resumed {
            br#"{"PolicyNames":["default"],"Truncated":false}"#.to_vec()
        } else {
            br#"{"PolicyNames":[],"Truncated":true,"NextMarker":"resume2"}"#.to_vec()
        };
    }
    let members = if case.child == "ListPolicyVersions" {
        if resumed {
            "<Versions><member><VersionId>v1</VersionId><IsDefaultVersion>true</IsDefaultVersion><CreateDate>2026-01-01T00:00:00Z</CreateDate></member></Versions>"
        } else {
            "<Versions/>"
        }
    } else if resumed {
        "<PolicyNames><member>shared</member></PolicyNames>"
    } else {
        "<PolicyNames/>"
    };
    xml(
        case.child,
        &format!(
            "{members}<IsTruncated>{}</IsTruncated>{}",
            if resumed { "false" } else { "true" },
            if resumed {
                ""
            } else {
                "<Marker>resume2</Marker>"
            }
        ),
    )
}
