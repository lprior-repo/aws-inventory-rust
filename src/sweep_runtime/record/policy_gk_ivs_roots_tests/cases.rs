use super::*;

pub(super) struct Case {
    pub operation: &'static str,
    pub items: &'static str,
    pub kind: &'static str,
    pub resource: &'static str,
}

pub(super) const CASES: &[Case] = &[
    Case {
        operation: "ListAdConfigurations",
        items: "adConfigurations",
        kind: "AWS::IVS::AdConfiguration",
        resource: "ad-configuration",
    },
    Case {
        operation: "ListFailoverSlateConfigurations",
        items: "failoverSlateConfigurations",
        kind: "AWS::ApiOnly::IVSFailoverSlateConfiguration",
        resource: "failover-slate-configuration",
    },
];

impl Case {
    pub fn id(&self, ordinal: &str) -> String {
        format!(
            "arn:aws:ivs:us-east-1:123456789012:{}/{ordinal}",
            self.resource
        )
    }

    pub fn row(&self, ordinal: &str) -> Json {
        let mut row = json!({"arn":self.id(ordinal), "name":format!("owned-{ordinal}"), "tags":{"owner":"inventory", "page":ordinal}});
        if self.operation == "ListAdConfigurations" {
            let Some(fields) = row.as_object_mut() else {
                return row;
            };
            fields.insert("mediaTailorPlaybackConfigurations".to_owned(), json!([
                {"playbackConfigurationArn":format!("arn:aws:mediatailor:us-east-1:123456789012:playbackConfiguration/{ordinal}")}
            ]));
            fields.insert("postRollConfiguration".to_owned(), json!({"durationSeconds":if ordinal == "first" {15} else {30}, "enabled":ordinal == "later"}));
        } else {
            let Some(fields) = row.as_object_mut() else {
                return row;
            };
            fields.insert(
                "activationThresholdSeconds".to_owned(),
                json!(if ordinal == "first" { 3 } else { 5 }),
            );
            fields.insert(
                "minSlateDisplaySeconds".to_owned(),
                json!(if ordinal == "first" { 3 } else { 10 }),
            );
            fields.insert(
                "maxSlateDisplaySeconds".to_owned(),
                json!(if ordinal == "first" { 60 } else { 180 }),
            );
            fields.insert(
                "state".to_owned(),
                json!(if ordinal == "first" {
                    "CREATING"
                } else {
                    "CREATED"
                }),
            );
        }
        row
    }

    pub fn page(&self, page: usize) -> TestResult<Json> {
        let (rows, token) = match page {
            0 => (Vec::new(), Some("second")),
            1 => (vec![self.row("first")], Some("third")),
            2 => (vec![self.row("later")], None),
            _ => return Err("unexpected IVS response page".into()),
        };
        let mut body = json!({self.items: rows});
        if let (Some(token), Some(fields)) = (token, body.as_object_mut()) {
            fields.insert("nextToken".to_owned(), json!(token));
        }
        Ok(body)
    }
}
