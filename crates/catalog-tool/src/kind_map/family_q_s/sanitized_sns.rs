use crate::kind_map::{ApiOnlyMapping, MappingSource, OperationMapping};

pub(super) const SOURCE: MappingSource<'static> = MappingSource {
    api_only: &[ApiOnlyMapping {
        kind: "AWS::ApiOnly::SNSPlatformApplication",
        reason: "SNS platform application ARN metadata; complete Attributes map excluded before decoding",
    }],
    operations: &[OperationMapping {
        service: "sns",
        operation: "ListPlatformApplications",
        kind: "AWS::ApiOnly::SNSPlatformApplication",
    }],
    ..super::super::EMPTY
};
