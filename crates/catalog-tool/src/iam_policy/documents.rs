use super::*;
use serde_json::json;

pub(super) const SIZE_LIMIT: usize = 6144;
pub(super) const ROLE_LIMIT: usize = 20;

fn document(actions: &[&str]) -> Result<String, Error> {
    Ok(serde_json::to_string(
        &json!({"Version":"2012-10-17", "Statement":[{
            "Effect":"Allow", "Action": actions, "Resource":"*"
        }]}),
    )?)
}

pub(super) fn split(actions: &BTreeSet<String>) -> Result<Vec<String>, Error> {
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
    if policies.len() > ROLE_LIMIT {
        return Err(Error::TooManyPolicies {
            count: policies.len(),
            limit: ROLE_LIMIT,
        });
    }
    Ok(policies)
}

pub(super) fn template(policies: &[String]) -> String {
    let resources: String = policies.iter().enumerate().map(|(index, policy)| {
        let number = index.saturating_add(1);
        format!("  InventoryReadOnlyPolicy{number}:\n    Type: AWS::IAM::ManagedPolicy\n    Properties:\n      Description: Generated catalog read-only actions.\n      PolicyDocument: {policy}\n")
    }).collect();
    let attachments: String = (1..=policies.len())
        .map(|number| format!("        - !Ref InventoryReadOnlyPolicy{number}\n"))
        .collect();
    include_str!("role.yaml.in")
        .replace("@POLICIES@\n", &resources)
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
        let policies = split(&actions)?;
        assert_eq!(policies.first().map(String::len), Some(SIZE_LIMIT));
        let mut more = actions;
        more.insert("z".into());
        assert_eq!(split(&more)?.len(), 2);
        Ok(())
    }
    #[test]
    fn given_role_maximum_when_one_policy_added_then_refused() -> Result<(), Error> {
        let actions = (0..ROLE_LIMIT)
            .map(|id| format!("{id:02}{}", "a".repeat(6000)))
            .collect();
        assert_eq!(split(&actions)?.len(), ROLE_LIMIT);
        let mut too_many = actions;
        too_many.insert("z".repeat(6000));
        assert!(split(&too_many).is_err());
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
        let policies = split(&exact)?;
        assert_eq!(policies.len(), 1);
        assert_eq!(policies.first().map(String::len), Some(SIZE_LIMIT));
        let overflow = BTreeSet::from(["a".repeat(padding.saturating_add(1)), "z".into()]);
        assert_eq!(split(&overflow)?.len(), 2);
        assert!(split(&BTreeSet::from(["a".repeat(6200)])).is_err());
        Ok(())
    }

    #[test]
    fn given_maximum_role_when_templated_then_it_fits_the_documented_s3_upload_limit()
    -> Result<(), Error> {
        let actions = (0..ROLE_LIMIT)
            .map(|id| format!("{id:02}{}", "a".repeat(6000)))
            .collect();
        let policies = split(&actions)?;
        assert_eq!(policies.len(), ROLE_LIMIT);
        assert!(template(&policies).len() <= 1_000_000);
        Ok(())
    }
}
