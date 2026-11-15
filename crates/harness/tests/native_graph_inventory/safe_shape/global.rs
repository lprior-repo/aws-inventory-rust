use super::Shape;

const TREE_0: &[(&str, Shape)] = &[
            ("", Shape::Map),
            ("Id", Shape::Text),
            ("Arn", Shape::Text),
            ("FeatureSet", Shape::Text),
            ("MasterAccountArn", Shape::Text),
            ("MasterAccountId", Shape::Text),
            ("AvailablePolicyTypes", Shape::List),
            ("AvailablePolicyTypes.*", Shape::Map),
            ("AvailablePolicyTypes.*.Type", Shape::Text),
            ("AvailablePolicyTypes.*.Status", Shape::Text),
        ];

const TREE_1: &[(&str, Shape)] = &[
            ("", Shape::Map),
            ("Id", Shape::Text),
            ("Name", Shape::Text),
            ("CallerReference", Shape::Text),
            ("Config", Shape::Map),
            ("Config.Comment", Shape::Text),
            ("Config.PrivateZone", Shape::Bool),
            ("ResourceRecordSetCount", Shape::Integer),
            ("LinkedService", Shape::Map),
            ("LinkedService.ServicePrincipal", Shape::Text),
            ("LinkedService.Description", Shape::Text),
            ("Features", Shape::Map),
            ("Features.AcceleratedRecoveryStatus", Shape::Text),
            ("Features.FailureReasons", Shape::Map),
            ("Features.FailureReasons.AcceleratedRecovery", Shape::Text),
        ];

const TREE_2: &[(&str, Shape)] = &[
            ("", Shape::Map),
            ("Id", Shape::Text),
            ("Description", Shape::Text),
            ("Name", Shape::Text),
            ("SigningProtocol", Shape::Text),
            ("SigningBehavior", Shape::Text),
            ("OriginAccessControlOriginType", Shape::Text),
        ];

pub(super) fn tree(kind: &str) -> Option<&'static [(&'static str, Shape)]> {
    match kind {
        "AWS::Organizations::Organization" => Some(TREE_0),
        "AWS::Route53::HostedZone" => Some(TREE_1),
        "AWS::CloudFront::OriginAccessControl" => Some(TREE_2),
        _ => None,
    }
}
