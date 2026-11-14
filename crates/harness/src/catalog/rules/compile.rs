use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::json::Json;
use crate::catalog::rules::model::{Descriptor, Rule};
use crate::catalog::rules::{
    ARN_PATH_OVERRIDES, EC2_IDENTIFIERS, IDENTIFIER_PATH_EXCLUSIONS, model::arn_target,
};
use crate::catalog::schema::{MAX_RELATIONSHIP_RULES, normalize, schema_nodes, string_node, under};

const ARN_SUFFIXES: &[(&str, &str, &str)] = &[
    ("secretarn", "USES_SECRET", "aws_secrets_manager_secret"),
    (
        "targetgrouparn",
        "USES_TARGET_GROUP",
        "aws_elbv2_target_group",
    ),
    (
        "loadbalancerarn",
        "USES_LOAD_BALANCER",
        "aws_elbv2_load_balancer",
    ),
    ("functionarn", "USES_FUNCTION", "aws_lambda_function"),
    ("topicarn", "USES_TOPIC", "aws_sns_topic"),
    ("queuearn", "USES_QUEUE", "aws_sqs_queue"),
];

struct Compile<'a> {
    descriptor: &'a Descriptor,
    nodes: BTreeMap<String, Value>,
    protected: Vec<&'a String>,
    readonly: BTreeSet<&'a str>,
    self_paths: BTreeSet<&'a str>,
    primary: BTreeSet<&'a str>,
    global_sensitive: &'a BTreeSet<String>,
}

fn path_override(type_name: &str, pointer: &str) -> Option<(&'static str, &'static str)> {
    ARN_PATH_OVERRIDES
        .iter()
        .find(|(name, path, _, _)| *name == type_name && *path == pointer)
        .map(|(_, _, relation, kind)| (*relation, *kind))
}

fn identifier_excluded(type_name: &str, pointer: &str) -> bool {
    IDENTIFIER_PATH_EXCLUSIONS
        .iter()
        .any(|(name, path)| *name == type_name && *path == pointer)
}

fn ec2_identifier(
    key: &str,
) -> Option<(&'static str, &'static str, &'static str, &'static str, bool)> {
    EC2_IDENTIFIERS
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| *value)
}

fn arn_semantics(field: &str) -> (&'static str, &'static str) {
    let key = normalize(field);
    if let Some(found) = role_semantics(&key) {
        return found;
    }
    if key.ends_with("kmskeyarn") || key.ends_with("encryptionkeyarn") || key == "kmskeyid" {
        return ("ENCRYPTED_BY", "aws_kms_key");
    }
    ARN_SUFFIXES
        .iter()
        .find(|(suffix, _, _)| key.ends_with(suffix))
        .map(|(_, relation, kind)| (*relation, *kind))
        .unwrap_or(("REFERENCES", "aws_reference"))
}

fn role_semantics(key: &str) -> Option<(&'static str, &'static str)> {
    if key.ends_with("executionrolearn") {
        return Some(("USES_EXECUTION_ROLE", "aws_iam_role"));
    }
    if key.ends_with("taskrolearn") {
        return Some(("USES_TASK_ROLE", "aws_iam_role"));
    }
    if key.ends_with("rolearn")
        || matches!(key, "role" | "servicerole" | "iamrole" | "executionrole")
    {
        return Some(("ASSUMES_ROLE", "aws_iam_role"));
    }
    None
}

pub fn enrich_rules(
    descriptor: &Descriptor,
    schema: &Value,
    global_sensitive: &BTreeSet<String>,
) -> Result<Vec<Rule>, HarnessError> {
    let compile = Compile::new(descriptor, schema, global_sensitive)?;
    let mut result = compile.reviewed_map()?;
    compile.extend_generated(&mut result)?;
    compile.finish(result)
}

impl<'a> Compile<'a> {
    fn new(
        descriptor: &'a Descriptor,
        schema: &'a Value,
        global_sensitive: &'a BTreeSet<String>,
    ) -> Result<Self, HarnessError> {
        let mut protected: Vec<&String> = descriptor.write_only_properties.iter().collect();
        protected.extend(descriptor.sensitive_properties.iter());
        Ok(Self {
            nodes: schema_nodes(schema)?,
            protected,
            readonly: readonly_paths(schema),
            self_paths: descriptor.arn_pointers.iter().map(String::as_str).collect(),
            primary: descriptor
                .primary_identifier
                .iter()
                .map(String::as_str)
                .collect(),
            descriptor,
            global_sensitive,
        })
    }

    fn reviewed_map(&self) -> Result<BTreeMap<String, Rule>, HarnessError> {
        let mut result = BTreeMap::new();
        for original in &self.descriptor.relationships {
            if let Some(rule) = self.reviewed_rule(original)? {
                result.insert(rule.pointer.clone(), rule);
            }
        }
        Ok(result)
    }

    fn reviewed_rule(&self, original: &Rule) -> Result<Option<Rule>, HarnessError> {
        let mut rule = original.clone();
        if !self.nodes.contains_key(&rule.pointer) {
            return Err(HarnessError::invalid(format!(
                "reviewed relationship is absent from schema: {} {}",
                self.descriptor.type_name, rule.pointer
            )));
        }
        if identifier_excluded(&self.descriptor.type_name, &rule.pointer) {
            return Ok(None);
        }
        if self.protected.iter().any(|path| under(&rule.pointer, path)) {
            return Ok(None);
        }
        apply_overrides(&mut rule, &self.descriptor.type_name);
        Ok(Some(rule))
    }

    fn extend_generated(&self, result: &mut BTreeMap<String, Rule>) -> Result<(), HarnessError> {
        for (path, node) in &self.nodes {
            if !string_node(node) || result.contains_key(path) {
                continue;
            }
            if let Some(rule) = self.generated_rule(path, node) {
                result.insert(path.clone(), rule);
            }
        }
        Ok(())
    }

    fn generated_rule(&self, path: &str, node: &Value) -> Option<Rule> {
        if !self.eligible(path) {
            return None;
        }
        let field = field_name(path);
        let key = normalize(field);
        if let Some(rule) = self.identifier_rule(path, &key) {
            return Some(rule);
        }
        self.semantic_rule(path, node, field, &key)
    }

    fn eligible(&self, path: &str) -> bool {
        if self.protected.iter().any(|prefix| under(path, prefix)) {
            return false;
        }
        if identifier_excluded(&self.descriptor.type_name, path) {
            return false;
        }
        !path
            .split('/')
            .skip(1)
            .any(|part| self.global_sensitive.contains(&normalize(part)))
    }

    fn identifier_rule(&self, path: &str, key: &str) -> Option<Rule> {
        let (relation, kind, resource, prefix, account_scoped) = ec2_identifier(key)?;
        if self.primary.contains(path) && self.descriptor.kind == kind {
            return None;
        }
        let account = if account_scoped { "{account}" } else { "" };
        let template =
            format!("arn:{{partition}}:ec2:{{region}}:{account}:{resource}/{{identifier}}");
        Some(Rule::generated(
            path,
            relation,
            kind,
            Some(&template),
            Some(prefix),
        ))
    }

    fn semantic_rule(&self, path: &str, node: &Value, field: &str, key: &str) -> Option<Rule> {
        let semantic =
            path_override(&self.descriptor.type_name, path).unwrap_or_else(|| arn_semantics(field));
        let described = node
            .get("pattern")
            .and_then(Value::as_str)
            .is_some_and(|pattern| pattern.to_lowercase().contains("arn:"))
            || node.get("format").and_then(Value::as_str) == Some("arn");
        let named = key.ends_with("arn") || key.ends_with("arns");
        if !(named || described || semantic.0 != "REFERENCES") {
            return None;
        }
        if self.self_paths.contains(path)
            || self.primary.contains(path)
            || (path.matches('/').count() == 1 && self.readonly.contains(path))
        {
            return None;
        }
        Some(Rule::generated(path, semantic.0, semantic.1, None, None))
    }

    fn finish(&self, result: BTreeMap<String, Rule>) -> Result<Vec<Rule>, HarnessError> {
        if result.len() > MAX_RELATIONSHIP_RULES {
            return Err(HarnessError::invalid(format!(
                "relationship descriptor budget exceeded: {}",
                self.descriptor.type_name
            )));
        }
        let mut rules: Vec<Rule> = result.into_values().collect();
        rules.sort_by(|left, right| {
            (left.pointer.as_str(), left.relation.as_str())
                .cmp(&(right.pointer.as_str(), right.relation.as_str()))
        });
        Ok(rules)
    }
}

fn field_name(path: &str) -> &str {
    let parts: Vec<&str> = path.split('/').skip(1).collect();
    parts
        .iter()
        .rev()
        .find(|part| **part != "*")
        .copied()
        .unwrap_or("")
}

fn readonly_paths(schema: &Value) -> BTreeSet<&str> {
    schema
        .get("readOnlyProperties")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(|value| value.strip_prefix("/properties"))
                .collect()
        })
        .unwrap_or_default()
}

fn apply_overrides(rule: &mut Rule, type_name: &str) {
    if let Some((relation, kind)) = path_override(type_name, &rule.pointer) {
        rule.relation = relation.to_string();
        rule.target_kind = kind.to_string();
    }
    if rule.target_format == "arn"
        && let Some((service, prefix)) = arn_target(&rule.target_kind)
    {
        rule.arn_service = Some(service.to_string());
        rule.arn_resource_prefix = prefix.map(str::to_string);
    }
    if rule.target_format == "template"
        && let Some((_, value)) = EC2_IDENTIFIERS
            .iter()
            .find(|(_, value)| value.1 == rule.target_kind)
    {
        rule.identifier_prefix = Some((value.3).to_string());
    }
}

pub fn rules_json(rules: &[Rule]) -> Json {
    Json::array(rules.iter().map(Rule::to_json).collect())
}
