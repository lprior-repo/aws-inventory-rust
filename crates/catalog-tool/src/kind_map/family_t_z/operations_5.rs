use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "timestream-influxdb",
        operation: "GetDbCluster",
        kind: "AWS::Timestream::InfluxDBCluster",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "GetDbInstance",
        kind: "AWS::Timestream::InfluxDBInstance",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "GetDbParameterGroup",
        kind: "AWS::ApiOnly::TimestreamInfluxDBParameterGroup",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeCertificate",
        kind: "AWS::Transfer::Certificate",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeConnector",
        kind: "AWS::Transfer::Connector",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeProfile",
        kind: "AWS::Transfer::Profile",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeServer",
        kind: "AWS::Transfer::Server",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeWebApp",
        kind: "AWS::Transfer::WebApp",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeWorkflow",
        kind: "AWS::Transfer::Workflow",
    },
    OperationMapping {
        service: "waf",
        operation: "GetByteMatchSet",
        kind: "AWS::WAF::ByteMatchSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetGeoMatchSet",
        kind: "AWS::ApiOnly::WAFGeoMatchSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetIPSet",
        kind: "AWS::WAF::IPSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetRateBasedRule",
        kind: "AWS::ApiOnly::WAFRateBasedRule",
    },
    OperationMapping {
        service: "waf",
        operation: "GetRegexMatchSet",
        kind: "AWS::ApiOnly::WAFRegexMatchSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetRegexPatternSet",
        kind: "AWS::ApiOnly::WAFRegexPatternSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetRuleGroup",
        kind: "AWS::ApiOnly::WAFRuleGroup",
    },
    OperationMapping {
        service: "waf",
        operation: "GetRule",
        kind: "AWS::WAF::Rule",
    },
    OperationMapping {
        service: "waf",
        operation: "GetSizeConstraintSet",
        kind: "AWS::WAF::SizeConstraintSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetSqlInjectionMatchSet",
        kind: "AWS::WAF::SqlInjectionMatchSet",
    },
    OperationMapping {
        service: "waf",
        operation: "GetWebACL",
        kind: "AWS::WAF::WebACL",
    },
];
