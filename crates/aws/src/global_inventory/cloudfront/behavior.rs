//! behavior responsibilities for cloudfront.
use super::*;

pub(super) struct Behavior<'a> {
    pub(super) path: &'a str,
    pub(super) target: &'a str,
    pub(super) viewer_protocol: &'a str,
    pub(super) cache_policy: Option<&'a str>,
    pub(super) origin_policy: Option<&'a str>,
    pub(super) response_policy: Option<&'a str>,
    pub(super) realtime_log: Option<&'a str>,
    pub(super) compress: Option<bool>,
    pub(super) allowed: &'a [Method],
    pub(super) lambdas: &'a [LambdaFunctionAssociation],
    pub(super) functions: &'a [FunctionAssociation],
    pub(super) minimum_ttl: Option<i64>,
    pub(super) default_ttl: Option<i64>,
    pub(super) maximum_ttl: Option<i64>,
}

pub(super) fn consume(used: &mut usize, count: usize) -> Result<(), AwsScanError> {
    *used = used.checked_add(count).ok_or(AwsScanError::ChildLimit)?;
    if *used > MAX_COMPONENTS {
        return Err(AwsScanError::InvalidData(
            "distribution exceeds 4096 mapped components".into(),
        ));
    }
    Ok(())
}

impl<'a> Behavior<'a> {
    #[expect(
        deprecated,
        reason = "Inventory retains legacy behavior TTL fields for distributions that do not use cache policies"
    )]
    pub(super) fn default(value: &'a DefaultCacheBehavior) -> Self {
        Self {
            path: "*",
            target: value.target_origin_id(),
            viewer_protocol: value.viewer_protocol_policy().as_str(),
            cache_policy: value.cache_policy_id(),
            origin_policy: value.origin_request_policy_id(),
            response_policy: value.response_headers_policy_id(),
            realtime_log: value.realtime_log_config_arn(),
            compress: value.compress(),
            allowed: value.allowed_methods().map(|v| v.items()).unwrap_or(&[]),
            lambdas: value
                .lambda_function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            functions: value
                .function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            minimum_ttl: value.min_ttl(),
            default_ttl: value.default_ttl(),
            maximum_ttl: value.max_ttl(),
        }
    }
    #[expect(
        deprecated,
        reason = "Inventory retains legacy behavior TTL fields for distributions that do not use cache policies"
    )]
    pub(super) fn ordered(value: &'a CacheBehavior) -> Self {
        Self {
            path: value.path_pattern(),
            target: value.target_origin_id(),
            viewer_protocol: value.viewer_protocol_policy().as_str(),
            cache_policy: value.cache_policy_id(),
            origin_policy: value.origin_request_policy_id(),
            response_policy: value.response_headers_policy_id(),
            realtime_log: value.realtime_log_config_arn(),
            compress: value.compress(),
            allowed: value.allowed_methods().map(|v| v.items()).unwrap_or(&[]),
            lambdas: value
                .lambda_function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            functions: value
                .function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            minimum_ttl: value.min_ttl(),
            default_ttl: value.default_ttl(),
            maximum_ttl: value.max_ttl(),
        }
    }
    pub(super) fn validate(&self, budget: &mut usize) -> Result<(), AwsScanError> {
        consume(budget, self.allowed.len())?;
        consume(budget, self.lambdas.len())?;
        consume(budget, self.functions.len())?;
        checked_text(self.target, "CloudFront behavior origin identifier")?;
        checked_text(self.path, "CloudFront behavior path")?;
        Ok(())
    }
    pub(super) fn properties(&self, origins: &BTreeSet<String>) -> Value {
        json!({"path_pattern":self.path,"target_origin_id":self.target,"target_in_configuration":origins.contains(self.target),
            "viewer_protocol_policy":self.viewer_protocol,"cache_policy_id":self.cache_policy,
            "origin_request_policy_id":self.origin_policy,"response_headers_policy_id":self.response_policy,
            "realtime_log_config_arn":self.realtime_log,"compress":self.compress,
            "allowed_methods":self.allowed.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
            "lambda_associations":self.lambdas.iter().map(|v| json!({"arn":v.lambda_function_arn(),"event_type":v.event_type().as_str(),"include_body":v.include_body()})).collect::<Vec<_>>(),
            "function_associations":self.functions.iter().map(|v| json!({"arn":v.function_arn(),"event_type":v.event_type().as_str()})).collect::<Vec<_>>(),
            "min_ttl":self.minimum_ttl,"default_ttl":self.default_ttl,"max_ttl":self.maximum_ttl,
            "forwarded_values_collected":false})
    }
}
