use super::*;
use regex::Regex;
use serde_json::json;

#[test]
fn policy_document_grants_read_actions_on_all_resources() -> TestResult {
    let document: Value =
        serde_json::from_str(&render(&reviewed_catalog()?, UnmappedPolicy::Refuse)?.text)?;
    assert_eq!(document.get("Version"), Some(&json!("2012-10-17")));
    assert_eq!(
        document
            .pointer("/Statement")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        document.pointer("/Statement/0/Effect"),
        Some(&json!("Allow"))
    );
    assert_eq!(document.pointer("/Statement/0/Resource"), Some(&json!("*")));
    Ok(())
}

#[test]
fn committed_policy_matches_the_reviewed_catalog() -> TestResult {
    run(&repo_root(), ReportMode::Check, UnmappedPolicy::Refuse)?;
    Ok(())
}

#[test]
fn deploy_template_grants_exactly_the_committed_policy_actions() -> TestResult {
    let template = fs::read_to_string(repo_root().join("deploy/read-only-role.yaml"))?;
    let template_actions = template_actions(&template)?;
    let policy = fs::read_to_string(repo_root().join("catalog/iam-read-only-policy.json"))?;
    let expected: BTreeSet<String> = policy_actions(&policy)?.into_iter().collect();
    let found: BTreeSet<String> = template_actions.iter().cloned().collect();
    assert!(!expected.is_empty());
    assert_eq!(
        template_actions.len(),
        found.len(),
        "duplicate template action"
    );
    assert_eq!(found, expected);
    assert!(found.iter().all(|action| !action.contains('*')));
    Ok(())
}

fn template_actions(template: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let action_line = Regex::new(r"^\s+- ([a-z0-9-]+:[A-Za-z0-9]+)$")?;
    Ok(template
        .lines()
        .filter_map(|line| {
            action_line
                .captures(line)
                .and_then(|found| found.get(1).map(|action| action.as_str().to_owned()))
        })
        .collect())
}
