//! Reviewable rule and descriptor types shared by both catalog builders.

use crate::catalog::json::Json;
use crate::catalog::rules::enrich_tables::ARN_TARGETS;

/// One emitted architecture reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Schema-relative property path.
    pub pointer: String,
    /// Semantic relation name.
    pub relation: String,
    /// Target resource kind.
    pub target_kind: String,
    /// `arn` or `template`.
    pub target_format: String,
    /// ARN template for `template` rules.
    pub arn_template: Option<String>,
    /// Evidence class for the rule.
    pub evidence: String,
    /// Identifier prefix for EC2 template rules.
    pub identifier_prefix: Option<String>,
    /// ARN service segment.
    pub arn_service: Option<String>,
    /// ARN resource-name prefix.
    pub arn_resource_prefix: Option<String>,
}

impl Rule {
    /// A reviewed seed rule; `arn` distinguishes template pointers.
    pub fn reviewed(path: &str, relation: &str, kind: &str, arn: Option<&str>) -> Self {
        Self {
            pointer: path.to_string(),
            relation: relation.to_string(),
            target_kind: kind.to_string(),
            target_format: if arn.is_some() { "template" } else { "arn" }.to_string(),
            arn_template: arn.map(str::to_string),
            evidence: "provider_property_reference".to_string(),
            identifier_prefix: None,
            arn_service: None,
            arn_resource_prefix: None,
        }
    }

    pub(super) fn generated(
        path: &str,
        relation: &str,
        kind: &str,
        template: Option<&str>,
        prefix: Option<&str>,
    ) -> Self {
        let mut rule = Self {
            pointer: path.to_string(),
            relation: relation.to_string(),
            target_kind: kind.to_string(),
            target_format: format_class(template),
            arn_template: template.map(str::to_string),
            evidence: evidence_class(template),
            identifier_prefix: prefix.map(str::to_string),
            arn_service: None,
            arn_resource_prefix: None,
        };
        attach_arn(&mut rule, template);
        rule
    }

    /// Serialize with the descriptor's declared key order.
    pub fn to_json(&self) -> Json {
        let mut entries = self.base_entries();
        self.push_optional(&mut entries);
        Json::object(entries)
    }

    fn base_entries(&self) -> Vec<(&'static str, Json)> {
        vec![
            ("pointer", Json::string(self.pointer.as_str())),
            ("relation", Json::string(self.relation.as_str())),
            ("target_kind", Json::string(self.target_kind.as_str())),
            ("target_format", Json::string(self.target_format.as_str())),
            (
                "arn_template",
                match &self.arn_template {
                    Some(value) => Json::string(value.as_str()),
                    None => Json::Null,
                },
            ),
            ("evidence", Json::string(self.evidence.as_str())),
        ]
    }

    fn push_optional(&self, entries: &mut Vec<(&'static str, Json)>) {
        if let Some(value) = &self.identifier_prefix {
            entries.push(("identifier_prefix", Json::string(value.as_str())));
        }
        if let Some(value) = &self.arn_service {
            entries.push(("arn_service", Json::string(value.as_str())));
        }
        if let Some(value) = &self.arn_resource_prefix {
            entries.push(("arn_resource_prefix", Json::string(value.as_str())));
        }
    }
}

fn attach_arn(rule: &mut Rule, template: Option<&str>) {
    if template.is_some() {
        return;
    }
    if let Some((service, resource_prefix)) = arn_target(&rule.target_kind) {
        rule.arn_service = Some(service.to_string());
        rule.arn_resource_prefix = resource_prefix.map(str::to_string);
    }
}

fn format_class(template: Option<&str>) -> String {
    if template.is_some() {
        "template"
    } else {
        "arn"
    }
    .to_string()
}

fn evidence_class(template: Option<&str>) -> String {
    if template.is_some() {
        "schema_ec2_identifier_reference"
    } else {
        "schema_arn_property_reference"
    }
    .to_string()
}

/// The descriptor subset relationship compilation depends on.
pub struct Descriptor {
    /// CloudFormation type name.
    pub type_name: String,
    /// Native resource kind.
    pub kind: String,
    /// Reviewed primary identifier pointers.
    pub primary_identifier: Vec<String>,
    /// Reviewed self-ARN pointers.
    pub arn_pointers: Vec<String>,
    /// Schema write-only pointers.
    pub write_only_properties: Vec<String>,
    /// Reviewed sensitive pointers.
    pub sensitive_properties: Vec<String>,
    /// Reviewed seed rules.
    pub relationships: Vec<Rule>,
}

pub(super) fn arn_target(kind: &str) -> Option<(&'static str, Option<&'static str>)> {
    ARN_TARGETS
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, value)| *value)
}
