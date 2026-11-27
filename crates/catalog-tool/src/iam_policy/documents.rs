use super::*;
use serde_json::json;

pub(super) const SIZE_LIMIT: usize = 6144;
pub(super) const ROLE_LIMIT: usize = 20;

fn document_with_statement(statement: serde_json::Value) -> Result<String, Error> {
    Ok(serde_json::to_string(
        &json!({"Version":"2012-10-17", "Statement":[statement]}),
    )?)
}

fn document(actions: &[&str]) -> Result<String, Error> {
    document_with_statement(json!({
        "Effect":"Allow",
        "Action": actions,
        "Resource":"*"
    }))
}

#[derive(Clone, Copy)]
enum SensitiveResourceMode<'a> {
    Concrete { partition: &'a str, account: &'a str },
    CloudFormation,
}

pub(super) fn sensitive_policy(
    actions: &BTreeSet<String>,
    partition: &str,
    account: &str,
) -> Result<String, Error> {
    render_sensitive_policy(
        actions,
        SensitiveResourceMode::Concrete { partition, account },
    )
}

pub(super) fn sensitive_template_policy(
    actions: &BTreeSet<String>,
) -> Result<String, Error> {
    render_sensitive_policy(actions, SensitiveResourceMode::CloudFormation)
}

fn render_sensitive_policy(
    actions: &BTreeSet<String>,
    mode: SensitiveResourceMode<'_>,
) -> Result<String, Error> {
    let statements: Vec<_> = actions
        .iter()
        .map(|action| {
            let resource = match (action.as_str(), mode) {
                (
                    "ec2:DescribeVpnConnections"
                    | "elasticloadbalancing:DescribeListeners"
                    | "elasticloadbalancing:DescribeRules"
                    | "iam:ListVirtualMFADevices",
                    _,
                ) => json!("*"),
                ("eks:DescribeCluster", SensitiveResourceMode::Concrete { partition, account }) => {
                    json!(format!("arn:{partition}:eks:*:{account}:cluster/*"))
                }
                ("lambda:ListVersionsByFunction", SensitiveResourceMode::Concrete { partition, account }) => {
                    json!(format!("arn:{partition}:lambda:*:{account}:function:*"))
                }
                ("eks:DescribeCluster", SensitiveResourceMode::CloudFormation) => {
                    json!({"Fn::Sub":"arn:${AWS::Partition}:eks:*:${AWS::AccountId}:cluster/*"})
                }
                ("lambda:ListVersionsByFunction", SensitiveResourceMode::CloudFormation) => {
                    json!({"Fn::Sub":"arn:${AWS::Partition}:lambda:*:${AWS::AccountId}:function:*"})
                }
                _ => return Err(Error::EmptyResourceScope {
                    action: action.clone(),
                }),
            };
            Ok(json!({
                "Effect":"Allow",
                "Action":action,
                "Resource":resource
            }))
        })
        .collect::<Result<_, Error>>()?;
    let policy = serde_json::to_string(&json!({
        "Version":"2012-10-17",
        "Statement":statements
    }))?;
    if policy.chars().count() > SIZE_LIMIT {
        return Err(Error::ActionTooLarge {
            action: "sensitive-read policy".into(),
            limit: SIZE_LIMIT,
        });
    }
    Ok(policy)
}

pub(super) fn split(
    actions: &BTreeSet<String>,
    include_sensitive_policy: bool,
) -> Result<Vec<String>, Error> {
    let base = document(&[])?.chars().count();
    let mut current = Vec::new();
    let mut size = base;
    let mut policies = Vec::new();
    actions.iter().try_for_each(|action| -> Result<(), Error> {
        let length = serde_json::to_string(action)?.chars().count();
        let extra = length.saturating_add(usize::from(!current.is_empty()));
        if base.saturating_add(length) > SIZE_LIMIT {
            return Err(Error::ActionTooLarge {
                action: action.clone(),
                limit: SIZE_LIMIT,
            });
        }
        if size.saturating_add(extra) > SIZE_LIMIT {
            policies.push(document(&current)?);
            current.clear();
            size = base;
        }
        size = size
            .saturating_add(length)
            .saturating_add(usize::from(!current.is_empty()));
        current.push(action.as_str());
        Ok(())
    })?;
    if !current.is_empty() {
        policies.push(document(&current)?);
    }
    let count = policies.len().saturating_add(usize::from(include_sensitive_policy));
    if count > ROLE_LIMIT {
        return Err(Error::TooManyPolicies {
            count,
            limit: ROLE_LIMIT,
        });
    }
    Ok(policies)
}

pub(super) fn template(policies: &[String], sensitive_policy: Option<&str>) -> String {
    let resources: String = policies
        .iter()
        .enumerate()
        .map(|(index, policy)| {
            let number = index.saturating_add(1);
            format!("  InventoryReadOnlyPolicy{number}:\n    Type: AWS::IAM::ManagedPolicy\n    Properties:\n      Description: Generated catalog read-only actions.\n      PolicyDocument: {policy}\n")
        })
        .collect();
    let sensitive_resource = sensitive_policy
        .map(|policy| {
            format!("  InventorySensitiveReadPolicy:\n    Type: AWS::IAM::ManagedPolicy\n    Condition: AttachSensitiveReadPolicy\n    Properties:\n      Description: Optional sensitive collector read actions; disabled by default.\n      PolicyDocument: {policy}\n")
        })
        .unwrap_or_default();
    let mut attachments: String = (1..=policies.len())
        .map(|number| format!("        - !Ref InventoryReadOnlyPolicy{number}\n"))
        .collect();
    let sensitive_parameter = if sensitive_policy.is_some() {
        "  AttachSensitiveReadPolicy:\n    Type: String\n    Default: 'false'\n    AllowedValues:\n      - 'true'\n      - 'false'\n    Description: Attach the separately generated sensitive collector-read policy.\n".to_owned()
    } else {
        String::new()
    };
    let conditions_section = if sensitive_policy.is_some() {
        "Conditions:\n  AttachSensitiveReadPolicy: !Equals [!Ref AttachSensitiveReadPolicy, 'true']\n".to_owned()
    } else {
        String::new()
    };
    if sensitive_policy.is_some() {
        attachments.push_str(
            "        - !If [AttachSensitiveReadPolicy, !Ref InventorySensitiveReadPolicy, !Ref 'AWS::NoValue']\n",
        );
    }
    include_str!("role.yaml.in")
        .replace("@SENSITIVE_PARAMETER@", &sensitive_parameter)
        .replace("@CONDITIONS_SECTION@", &conditions_section)
        .replace("@POLICIES@\n", &format!("{resources}{sensitive_resource}"))
        .replace("@ATTACHMENTS@\n", &attachments)
        .replace("@POLICY_COUNT@", &policies.len().to_string())
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn given_exact_limit_when_another_action_added_then_split()
    -> Result<(), Box<dyn std::error::Error>> {
        let base = document(&[])?.len();
        let padding = SIZE_LIMIT
            .checked_sub(base)
            .and_then(|size| size.checked_sub(2))
            .ok_or("invalid test base")?;
        let actions = BTreeSet::from(["a".repeat(padding)]);
        let policies = split(&actions, false)?;
        assert_eq!(policies.first().map(String::len), Some(SIZE_LIMIT));
        let mut more = actions;
        more.insert("z".into());
        assert_eq!(split(&more, false)?.len(), 2);
        Ok(())
    }
    #[test]
    fn given_role_maximum_when_one_policy_added_then_refused() -> Result<(), Error> {
        let actions: BTreeSet<_> = (0..ROLE_LIMIT - 1)
            .map(|id| format!("{id:02}{}", "a".repeat(6000)))
            .collect();
        assert_eq!(split(&actions, false)?.len(), ROLE_LIMIT - 1);
        let mut too_many = actions;
        too_many.insert(format!("{}{}", ROLE_LIMIT, "a".repeat(6000)));
        assert_eq!(split(&too_many, false)?.len(), ROLE_LIMIT);
        assert!(matches!(
            split(&too_many, true),
            Err(Error::TooManyPolicies { count, limit: ROLE_LIMIT }) if count == ROLE_LIMIT + 1
        ));
        Ok(())
    }

    #[test]
    fn given_two_actions_when_comma_reaches_boundary_then_split_only_above_limit()
    -> Result<(), Box<dyn std::error::Error>> {
        let padding = SIZE_LIMIT
            .checked_sub(document(&[])?.len())
            .and_then(|n| n.checked_sub(6))
            .ok_or("invalid test base")?;
        let exact = BTreeSet::from(["a".repeat(padding), "z".into()]);
        let policies = split(&exact, false)?;
        assert_eq!(policies.len(), 1);
        assert_eq!(policies.first().map(String::len), Some(SIZE_LIMIT));
        let overflow = BTreeSet::from(["a".repeat(padding.saturating_add(1)), "z".into()]);
        assert_eq!(split(&overflow, false)?.len(), 2);
        assert!(split(&BTreeSet::from(["a".repeat(6200)]), false).is_err());
        Ok(())
    }

    #[test]
    fn given_sensitive_actions_when_rendered_then_only_six_verified_reads_are_in_policy()
    -> Result<(), Error> {
        let actions: BTreeSet<String> =
            super::super::SENSITIVE_ACTIONS.iter().map(|action| (*action).to_owned()).collect();
        let expected = BTreeSet::from([
            "ec2:DescribeVpnConnections".to_owned(),
            "eks:DescribeCluster".to_owned(),
            "elasticloadbalancing:DescribeListeners".to_owned(),
            "elasticloadbalancing:DescribeRules".to_owned(),
            "iam:ListVirtualMFADevices".to_owned(),
            "lambda:ListVersionsByFunction".to_owned(),
        ]);
        assert_eq!(actions, expected);
        let policy = sensitive_policy(&actions, "aws-us-gov", "123456789012")?;
        assert_eq!(actions.len(), 6);
        for action in &actions {
            assert!(policy.contains(action));
        }
        assert_eq!(policy.matches("\"Resource\":\"*\"").count(), 4);
        for resource in [
            "arn:aws-us-gov:eks:*:123456789012:cluster/*",
            "arn:aws-us-gov:lambda:*:123456789012:function:*",
        ] {
            assert!(policy.contains(resource), "{resource}");
        }
        let template_policy = sensitive_template_policy(&actions)?;
        assert!(!policy.contains("${AWS::Partition}"));
        assert!(!policy.contains("cloudfront:"));
        let role = template(&[], Some(&template_policy));
        assert!(role.contains("AttachSensitiveReadPolicy: !Equals"));
        assert!(role.contains("!If [AttachSensitiveReadPolicy"));
        assert!(role.contains("PolicyDocument: {"));
        assert!(role.contains("\"Fn::Sub\":\"arn:${AWS::Partition}:eks:*:${AWS::AccountId}:cluster/*\""));
        assert!(role.contains("\"Fn::Sub\":\"arn:${AWS::Partition}:lambda:*:${AWS::AccountId}:function:*\""));
        Ok(())
    }

    #[test]
    fn given_no_sensitive_actions_when_rendered_then_no_policy_or_attachment_is_generated()
    -> Result<(), Error> {
        let actions = BTreeSet::new();
        let policy = sensitive_policy(&actions, "aws-us-gov", "123456789012")?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&policy)?,
            json!({"Version":"2012-10-17", "Statement":[]})
        );
        let policies = vec![document(&["s3:ListBuckets"])?];
        let role = template(&policies, None);
        assert!(role.contains("!Ref InventoryReadOnlyPolicy1"));
        assert!(!role.contains("InventorySensitiveReadPolicy"));
        assert!(!role.contains("AttachSensitiveReadPolicy"));
        assert!(!role.contains("Conditions:"));
        Ok(())
    }

    #[test]
    fn given_maximum_role_when_templated_then_it_fits_the_documented_s3_upload_limit()
    -> Result<(), Error> {
        let actions = (0..ROLE_LIMIT - 1)
            .map(|id| format!("{id:02}{}", "a".repeat(6000)))
            .collect();
        let policies = split(&actions, true)?;
        assert_eq!(policies.len(), ROLE_LIMIT - 1);
        assert!(template(&policies, Some("{}")).len() <= 1_000_000);
        Ok(())
    }
}
