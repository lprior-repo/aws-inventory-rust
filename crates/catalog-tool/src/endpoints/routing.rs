use super::{Endpoint, Error, Fault, Outcome, RuleSet};
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl RuleSet {
    pub fn operation_endpoint(
        &self,
        region: &str,
        context: &BTreeMap<String, Value>,
    ) -> Result<Endpoint, Error> {
        if context.len() > 16 || context.contains_key("Endpoint") {
            return Err(self.error(Fault::Bound("operation endpoint context")));
        }
        let bindings = [
            ("Region".to_owned(), json!(region)),
            ("UseFIPS".to_owned(), json!(false)),
            ("UseDualStack".to_owned(), json!(false)),
        ]
        .into_iter()
        .chain(
            context
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        )
        .filter(|(name, _)| self.document.parameters.contains_key(name))
        .collect();
        match self.evaluate(&bindings)? {
            Outcome::Endpoint(endpoint) => Ok(endpoint),
            Outcome::Error(reason) => Err(self.error(Fault::Malformed(reason))),
            Outcome::UnsupportedFunction { name } => {
                Err(self.error(Fault::UnsupportedFunction { name }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_context_selects_modeled_outposts_signing() -> Result<(), Box<dyn std::error::Error>>
    {
        let rules = RuleSet::parse(
            std::path::Path::new("fixture"),
            &json!({"version":"1.0", "parameters":{"Region":{"type":"String","required":true},"OutpostId":{"type":"String"}},"rules":[{"conditions":[{"fn":"isSet","argv":[{"ref":"OutpostId"}]}],"type":"endpoint","endpoint":{"url":"https://s3-outposts.{Region}.amazonaws.com","properties":{"authSchemes":[{"name":"sigv4","signingName":"s3-outposts","signingRegion":"{Region}"}]}}}]}),
        )?;
        let endpoint = rules.operation_endpoint(
            "us-west-2",
            &BTreeMap::from([("OutpostId".to_owned(), json!("op-123"))]),
        )?;
        assert_eq!(endpoint.url, "https://s3-outposts.us-west-2.amazonaws.com");
        assert_eq!(
            endpoint.properties.pointer("/authSchemes/0/signingName"),
            Some(&json!("s3-outposts"))
        );
        Ok(())
    }
}
