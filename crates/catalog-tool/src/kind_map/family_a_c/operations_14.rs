use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "cloudwatch-logs", operation: "DescribeLookupTables", kind: "AWS::ApiOnly::LogsLookupTable" },
    OperationMapping { service: "cloudwatch-logs", operation: "DescribeMetricFilters", kind: "AWS::Logs::MetricFilter" },
    OperationMapping { service: "cloudwatch-logs", operation: "DescribeResourcePolicies", kind: "AWS::Logs::ResourcePolicy" },
    OperationMapping { service: "cloudwatch-logs", operation: "ListIntegrations", kind: "AWS::Logs::Integration" },
    OperationMapping { service: "cloudwatch-logs", operation: "ListLogAnomalyDetectors", kind: "AWS::Logs::LogAnomalyDetector" },
    OperationMapping { service: "cloudwatch-logs", operation: "ListScheduledQueries", kind: "AWS::Logs::ScheduledQuery" },
    OperationMapping { service: "cloudwatchomni", operation: "GetIntelligenceConfiguration", kind: "AWS::ApiOnly::CloudWatchOmniIntelligenceConfiguration" },
    OperationMapping { service: "cloudwatchomni", operation: "ListAccessGrants", kind: "AWS::ApiOnly::CloudWatchOmniAccessGrant" },
    OperationMapping { service: "cloudwatchomni", operation: "ListDomains", kind: "AWS::ApiOnly::CloudWatchOmniDomain" },
    OperationMapping { service: "cloudwatchomni", operation: "ListIntegrations", kind: "AWS::ApiOnly::CloudWatchOmniIntegration" },
    OperationMapping { service: "cloudwatchomni", operation: "ListSpaces", kind: "AWS::ApiOnly::CloudWatchOmniSpace" },
    OperationMapping { service: "cloudwatchomni", operation: "ListViews", kind: "AWS::ApiOnly::CloudWatchOmniView" },
    OperationMapping { service: "codeartifact", operation: "DescribeDomain", kind: "AWS::CodeArtifact::Domain" },
    OperationMapping { service: "codeartifact", operation: "DescribeRepository", kind: "AWS::CodeArtifact::Repository" },
    OperationMapping { service: "codeartifact", operation: "ListDomains", kind: "AWS::CodeArtifact::Domain" },
    OperationMapping { service: "codeartifact", operation: "ListRepositories", kind: "AWS::CodeArtifact::Repository" },
    OperationMapping { service: "codebuild", operation: "ListBuildBatches", kind: "AWS::ApiOnly::CodeBuildBuildBatch" },
    OperationMapping { service: "codebuild", operation: "ListBuilds", kind: "AWS::ApiOnly::CodeBuildBuild" },
    OperationMapping { service: "codebuild", operation: "ListFleets", kind: "AWS::CodeBuild::Fleet" },
];
