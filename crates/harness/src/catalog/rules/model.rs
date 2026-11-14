use crate::catalog::json::Json;
use crate::catalog::rules::enrich_tables::ARN_TARGETS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub pointer: String,

    pub relation: String,

    pub target_kind: String,

    pub target_format: String,

    pub arn_template: Option<String>,

    pub evidence: String,

    pub identifier_prefix: Option<String>,

    pub arn_service: Option<String>,

    pub arn_resource_prefix: Option<String>,
}

impl Rule {
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

pub struct Descriptor {
    pub type_name: String,

    pub kind: String,

    pub primary_identifier: Vec<String>,

    pub arn_pointers: Vec<String>,

    pub write_only_properties: Vec<String>,

    pub sensitive_properties: Vec<String>,

    pub relationships: Vec<Rule>,
}

pub(super) fn arn_target(kind: &str) -> Option<(&'static str, Option<&'static str>)> {
    ARN_TARGETS
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, value)| *value)
}
