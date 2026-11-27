use super::{fixture_endpoint, fixture_kinds, fixture_root, iam_policy};
use catalog_tool::{overlay, runtime_catalog, smithy};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    process::Command,
};

const ACCOUNT: &str = "210987654321";
const PARTITION: &str = "aws-us-gov";
const SENSITIVE_ACTIONS: [&str; 6] = [
    "ec2:DescribeVpnConnections",
    "eks:DescribeCluster",
    "elasticloadbalancing:DescribeListeners",
    "elasticloadbalancing:DescribeRules",
    "iam:ListVirtualMFADevices",
    "lambda:ListVersionsByFunction",
];
const OPERATIONS: [&str; 6] = [
    "DescribeVpnConnections",
    "DescribeCluster",
    "DescribeListeners",
    "DescribeRules",
    "ListVirtualMFADevices",
    "ListVersionsByFunction",
];
const REFERENCE: &str = "# Retrieved: 2026-11-24\nec2\tDescribeVpnConnections\tread\neks\tDescribeCluster\tread\nelasticloadbalancing\tDescribeListeners\tread\nelasticloadbalancing\tDescribeRules\tread\niam\tListVirtualMFADevices\tread\nlambda\tListVersionsByFunction\tread\n";
const MAPPINGS: &str = "[operation]\n\
\"ec2/DescribeVpnConnections\" = \"ec2:DescribeVpnConnections\"\n\
\"ec2/DescribeCluster\" = \"eks:DescribeCluster\"\n\
\"ec2/DescribeListeners\" = \"elasticloadbalancing:DescribeListeners\"\n\
\"ec2/DescribeRules\" = \"elasticloadbalancing:DescribeRules\"\n\
\"ec2/ListVirtualMFADevices\" = \"iam:ListVirtualMFADevices\"\n\
\"ec2/ListVersionsByFunction\" = \"lambda:ListVersionsByFunction\"\n";

#[test]
fn given_six_catalog_reads_when_written_then_only_opt_in_policy_grants_them()
-> Result<(), Box<dyn Error>> {
    let root = fixture_root()?;
    fs::write(root.path().join("catalog/aws-api.catalog"), catalog_bytes()?)?;
    fs::write(root.path().join(iam_policy::MAPPINGS_PATH), MAPPINGS)?;
    fs::write(root.path().join(iam_policy::REFERENCE_PATH), REFERENCE)?;
    fs::write(root.path().join(iam_policy::EXCEPTIONS_PATH), "")?;

    let output = Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args([
            "iam-policy",
            "--partition",
            PARTITION,
            "--account",
            ACCOUNT,
            "--root",
        ])
        .arg(root.path())
        .arg("--write")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "catalog-tool iam-policy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let default: Value = serde_json::from_slice(&fs::read(
        root.path().join("catalog/iam-read-only-policy.json"),
    )?)?;
    let standalone: Value = serde_json::from_slice(&fs::read(
        root.path().join("catalog/iam-sensitive-read-policy.json"),
    )?)?;
    let template = fs::read_to_string(root.path().join("deploy/read-only-role.yaml"))?;
    assert_grants_are_separated(&default, &standalone)?;
    assert_template_attachment(&template, &default)?;
    assert_scoped_standalone_policy(&standalone, &template)?;
    assert_policy_quotas(&default, &standalone, &template)?;
    Ok(())
}

fn catalog_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("models/ec2/service/2020-01-01");
    fs::create_dir_all(&source)?;
    let mut model = json!({
        "smithy":"2.0",
        "shapes":{
            "n#Service":{
                "type":"service",
                "version":"1",
                "traits":{
                    "aws.api#service":{
                        "sdkId":"EC2",
                        "endpointPrefix":"ec2",
                        "arnNamespace":"ec2"
                    },
                    "aws.auth#sigv4":{"name":"ec2"},
                    "aws.protocols#restJson1":{}
                }
            },
            "n#Output":{
                "type":"structure",
                "members":{"Items":{"target":"n#Items"}}
            },
            "n#Items":{"type":"list","member":{"target":"n#Item"}},
            "n#Item":{
                "type":"structure",
                "members":{"Id":{"target":"smithy.api#String"}}
            }
        }
    });
    let shapes = model
        .pointer_mut("/shapes")
        .and_then(Value::as_object_mut)
        .ok_or("fixture shapes absent")?;
    shapes.extend(OPERATIONS.iter().map(|name| {
        (
            format!("n#{name}"),
            json!({
                "type":"operation",
                "output":{"target":"n#Output"},
                "traits":{"smithy.api#http":{"method":"GET","uri":"/fixture"}}
            }),
        )
    }));
    fs::write(source.join("model.json"), serde_json::to_vec(&model)?)?;

    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.paginator = Some(overlay::Paginator::None);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new("Id")?]));
    let policies = [overlay::ServiceOverlay::new(
        "ec2",
        OPERATIONS.iter().map(|name| (*name, plan.clone())).collect(),
    )?];
    let checked = overlay::check_policies(
        directory.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse("Id\nItems\n")?;
    let base = smithy::build_catalog(directory.path(), &fields, "iam-contract-fixture")?;
    let kinds = fixture_kinds(&OPERATIONS)?;
    let endpoints = std::collections::BTreeMap::from([("ec2".to_owned(), fixture_endpoint()?)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}

fn assert_grants_are_separated(default: &Value, standalone: &Value) -> Result<(), Box<dyn Error>> {
    let expected = SENSITIVE_ACTIONS
        .iter()
        .map(|action| (*action).to_owned())
        .collect::<BTreeSet<_>>();
    let default_actions = actions(default)?;
    let opt_in_actions = actions(standalone)?;
    assert!(
        expected.is_disjoint(&default_actions),
        "sensitive reads leaked into the default policy"
    );
    assert_eq!(
        default_actions,
        BTreeSet::new(),
        "fixture has only sensitive operations; default grants must remain empty",
    );
    assert_eq!(
        opt_in_actions, expected,
        "opt-in policy must contain only the six intended reads"
    );
    Ok(())
}

fn assert_wildcard_resources(policy: &Value) -> Result<(), Box<dyn Error>> {
    [
        "ec2:DescribeVpnConnections",
        "elasticloadbalancing:DescribeListeners",
        "elasticloadbalancing:DescribeRules",
        "iam:ListVirtualMFADevices",
    ]
    .iter()
    .try_for_each(|action| {
        let statement = statement_for(policy, action)?;
        assert_eq!(
            statement.get("Resource"),
            Some(&Value::String("*".to_owned())),
            "{action} must use Resource *"
        );
        Ok::<(), Box<dyn Error>>(())
    })
}

fn assert_scoped_resource(
    policy: &Value,
    action: &str,
    resource: &str,
) -> Result<(), Box<dyn Error>> {
    let statement = statement_for(policy, action)?;
    assert_eq!(statement.get("Resource"), Some(&Value::String(resource.to_owned())));
    Ok(())
}
fn assert_template_resource_substitution(
    policy: &Value,
    action: &str,
    resource: &str,
) -> Result<(), Box<dyn Error>> {
    let statement = statement_for(policy, action)?;
    assert_eq!(statement.get("Resource"), Some(&json!({"Fn::Sub": resource})));
    Ok(())
}

fn assert_template_attachment(
    template: &str,
    default: &Value,
) -> Result<(), Box<dyn Error>> {
    assert!(
        template.contains("Default: 'false'"),
        "sensitive attachment must default off"
    );
    assert!(template.lines().any(|line| {
        line.trim()
            == "AttachSensitiveReadPolicy: !Equals [!Ref AttachSensitiveReadPolicy, 'true']"
    }));
    assert!(template.lines().any(|line| {
        line.trim() == "Condition: AttachSensitiveReadPolicy"
    }), "sensitive managed policy must carry its condition");
    let sensitive_attachments = template
        .lines()
        .filter(|line| {
            line.trim()
                == "- !If [AttachSensitiveReadPolicy, !Ref InventorySensitiveReadPolicy, !Ref 'AWS::NoValue']"
        })
        .count();
    assert_eq!(sensitive_attachments, 1);
    assert!(!template.lines().any(|line| {
        line.trim() == "- !Ref InventorySensitiveReadPolicy"
    }), "sensitive policy must not be attached unconditionally");

    let documents = template_documents(template)?;
    let default_documents = documents
        .iter()
        .filter(|(resource, _)| resource.starts_with("InventoryReadOnlyPolicy"))
        .map(|(_, policy)| policy.clone())
        .collect::<Vec<_>>();
    assert_eq!(default.as_array(), Some(&default_documents));
    let attached_defaults = template
        .lines()
        .filter(|line| {
            line.trim()
                .strip_prefix("- !Ref InventoryReadOnlyPolicy")
                .is_some()
        })
        .count();
    assert_eq!(attached_defaults, default_documents.len());

    let conditional = documents
        .into_iter()
        .find(|(resource, _)| resource == "InventorySensitiveReadPolicy")
        .map(|(_, policy)| policy)
        .ok_or("conditional sensitive policy document absent")?;
    let actions = actions(&conditional)?;
    assert_eq!(
        actions,
        SENSITIVE_ACTIONS.iter().map(|action| (*action).to_owned()).collect(),
        "conditional role policy must match the standalone opt-in policy",
    );
    let statements = conditional
        .get("Statement")
        .and_then(Value::as_array)
        .ok_or("conditional policy statements absent")?;
    assert_eq!(statements.len(), SENSITIVE_ACTIONS.len());
    Ok(())
}

fn assert_scoped_standalone_policy(
    standalone: &Value,
    template: &str,
) -> Result<(), Box<dyn Error>> {
    assert!(!standalone.to_string().contains("${AWS::"));
    assert_wildcard_resources(standalone)?;
    assert_scoped_resource(
        standalone,
        "eks:DescribeCluster",
        &format!("arn:{PARTITION}:eks:*:{ACCOUNT}:cluster/*"),
    )?;
    assert_scoped_resource(
        standalone,
        "lambda:ListVersionsByFunction",
        &format!("arn:{PARTITION}:lambda:*:{ACCOUNT}:function:*"),
    )?;
    let template_sensitive = template_documents(template)?
        .into_iter()
        .find(|(resource, _)| resource == "InventorySensitiveReadPolicy")
        .map(|(_, policy)| policy)
        .ok_or("template sensitive policy absent")?;
    assert_template_policy_shape(&template_sensitive)?;
    assert_wildcard_resources(&template_sensitive)?;
    assert_template_resource_substitution(
        &template_sensitive,
        "eks:DescribeCluster",
        "arn:${AWS::Partition}:eks:*:${AWS::AccountId}:cluster/*",
    )?;
    assert_template_resource_substitution(
        &template_sensitive,
        "lambda:ListVersionsByFunction",
        "arn:${AWS::Partition}:lambda:*:${AWS::AccountId}:function:*",
    )?;
    Ok(())
}

fn assert_template_policy_shape(policy: &Value) -> Result<(), Box<dyn Error>> {
    let object = policy.as_object().ok_or("PolicyDocument is not an object")?;
    let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(keys, BTreeSet::from(["Statement", "Version"]));
    assert_eq!(
        policy.get("Version"),
        Some(&Value::String("2012-10-17".to_owned()))
    );
    let statements = policy
        .get("Statement")
        .and_then(Value::as_array)
        .ok_or("PolicyDocument statements absent")?;
    statements.iter().try_for_each(|statement| {
        let object = statement
            .as_object()
            .ok_or("PolicyDocument statement is not an object")?;
        let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
        assert_eq!(keys, BTreeSet::from(["Action", "Effect", "Resource"]));
        assert_eq!(
            statement.get("Effect"),
            Some(&Value::String("Allow".to_owned()))
        );
        Ok::<(), Box<dyn Error>>(())
    })?;
    Ok(())
}


fn assert_policy_quotas(
    default: &Value,
    standalone: &Value,
    template: &str,
) -> Result<(), Box<dyn Error>> {
    const DOCUMENT_LIMIT: usize = 6_144;
    const ROLE_LIMIT: usize = 20;
    let default_documents = default
        .as_array()
        .ok_or("default policy bundle is not an array")?;
    default_documents.iter().try_for_each(|document| {
        assert!(serde_json::to_vec(document)?.len() <= DOCUMENT_LIMIT);
        Ok::<(), Box<dyn Error>>(())
    })?;
    assert!(serde_json::to_vec(standalone)?.len() <= DOCUMENT_LIMIT);
    let attached = template
        .lines()
        .filter(|line| {
            let line = line.trim();
            line.starts_with("- !Ref InventoryReadOnlyPolicy")
                || line.starts_with(
                    "- !If [AttachSensitiveReadPolicy, !Ref InventorySensitiveReadPolicy,",
                )
        })
        .count();
    assert_eq!(attached, default_documents.len().saturating_add(1));
    assert!(attached <= ROLE_LIMIT);
    Ok(())
}

pub(super) fn template_documents(template: &str) -> Result<Vec<(String, Value)>, Box<dyn Error>> {
    let mut resource = String::new();
    template.lines().try_fold(Vec::new(), |mut documents, line| {
        if line.starts_with("  ") && !line.starts_with("    ") && line.ends_with(':') {
            resource = line
                .strip_prefix("  ")
                .and_then(|name| name.strip_suffix(':'))
                .ok_or("invalid resource declaration")?
                .to_owned();
        }
        if let Some(encoded) = line.trim().strip_prefix("PolicyDocument: ") {
            documents.push((resource.clone(), parse_cfn_policy(encoded)?));
        }
        Ok::<_, Box<dyn Error>>(documents)
    })
}

fn parse_cfn_policy(encoded: &str) -> Result<Value, Box<dyn Error>> {
    if !encoded.starts_with('{') {
        return Err("PolicyDocument must be an object-valued JSON flow mapping".into());
    }
    let policy: Value = serde_json::from_str(encoded)?;
    if !policy.is_object() {
        return Err("PolicyDocument must be an object".into());
    }
    Ok(policy)
}

fn actions(policy: &Value) -> Result<BTreeSet<String>, Box<dyn Error>> {
    match policy {
        Value::Array(documents) => documents.iter().try_fold(
            BTreeSet::new(),
            |mut names, document| {
                names.extend(document_actions(document)?);
                Ok(names)
            },
        ),
        Value::Object(_) => document_actions(policy),
        _ => Err("policy must be an object or array".into()),
    }
}

fn document_actions(document: &Value) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let statements = document
        .get("Statement")
        .and_then(Value::as_array)
        .ok_or("policy statements absent")?;
    statements.iter().try_fold(BTreeSet::new(), |mut names, statement| {
        let action = statement.get("Action").ok_or("statement Action absent")?;
        match action {
            Value::String(name) => {
                names.insert(name.clone());
            }
            Value::Array(values) => values.iter().try_for_each(|value| {
                names.insert(value.as_str().ok_or("non-string IAM action")?.to_owned());
                Ok::<(), Box<dyn Error>>(())
            })?,
            _ => return Err("unsupported IAM Action shape".into()),
        }
        Ok(names)
    })
}

fn statement_for<'a>(policy: &'a Value, action: &str) -> Result<&'a Value, Box<dyn Error>> {
    policy
        .get("Statement")
        .and_then(Value::as_array)
        .and_then(|statements| statements.iter().find(|statement| {
            statement.get("Action").is_some_and(|actions| {
                actions.as_str() == Some(action)
                    || actions.as_array().is_some_and(|values| {
                        values.iter().any(|value| value.as_str() == Some(action))
                    })
            })
        }))
        .ok_or_else(|| format!("statement for {action} absent").into())
}
