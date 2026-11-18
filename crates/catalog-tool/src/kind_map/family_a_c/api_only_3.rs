use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CleanRoomsMLAudienceModel",
        reason: "Clean Rooms ML trained audience model metadata is not an audience-generation job or configured audience model; no exact pinned type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CleanRoomsMLConfiguredAudienceModel",
        reason: "Clean Rooms ML audience-model sharing/configuration metadata is distinct from its trained audience model; no exact pinned type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CleanRoomsMLConfiguredModelAlgorithm",
        reason: "Clean Rooms ML model-algorithm registration identifies algorithm access configuration, not a SageMaker algorithm or training execution.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CleanRoomsMLTrainingDataset",
        reason: "Clean Rooms ML training-dataset metadata is a registered training input configuration, not underlying S3 content or a Glue table definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Cloud9Environment",
        reason: "ListEnvironments can return both EC2 and SSH environments; mapping all IDs to AWS::Cloud9::EnvironmentEC2 would falsely identify SSH environments.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudDirectoryDirectory",
        reason: "Cloud Directory directory metadata keyed by DirectoryArn is not a Directory Service managed directory; no pinned CFN CloudDirectory directory type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudDirectorySchema",
        reason: "Cloud Directory development/published schema ARNs identify directory schemas, not directory objects; no pinned CFN CloudDirectory schema type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudFormationGeneratedTemplate",
        reason: "An existing generated-template metadata object is an IaC generation artifact, not an actual deployed CloudFormation stack; template body is never retrieved.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudFormationResourceScan",
        reason: "A CloudFormation resource-scan summary identifies an existing discovery execution, not a stack or resource type definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudHSMClient",
        reason: "CloudHSM Classic Luna client metadata keyed by ClientArn is distinct from an HSM or cluster; no exact pinned CFN Classic client type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudHSMHAPG",
        reason: "CloudHSM Classic high-availability partition-group metadata keyed by HapgArn is not a current CloudHSM v2 cluster; no exact pinned CFN Classic HAPG type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudHSMHSM",
        reason: "CloudHSM Classic HSM metadata keyed by HsmArn is not a current CloudHSM v2 cluster; no exact pinned CFN Classic HSM type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudHSMV2Backup",
        reason: "CloudHSM v2 backup metadata is a recovery snapshot, not a live CloudHSM cluster or hardware HSM; no exact pinned backup type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudSearchDomain",
        reason: "CloudSearch search-domain configuration keyed by ARN is distinct from OpenSearch domains; the pinned CFN set does not contain the legacy CloudSearch domain type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudTrailImport",
        reason: "A retained CloudTrail Lake import summary is an existing ingestion execution, not an event data store or trail definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniAccessGrant",
        reason: "CloudWatch Omni access-grant metadata identifies domain permissions, not an IAM role or S3 AccessGrant; no matching pinned Omni grant type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniDomain",
        reason: "CloudWatch Omni domain configuration metadata is not an OpenSearch domain or CloudWatch dashboard; no matching pinned Omni domain type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniIntegration",
        reason: "CloudWatch Omni integration summaries contain credential ARNs as references rather than credentials; the integration is not an AppIntegrations resource or IAM role.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniIntelligenceConfiguration",
        reason: "CloudWatch Omni account intelligence encryption settings are an account singleton, not an Omni domain or KMS key resource definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::CloudWatchOmniSpace",
        reason: "CloudWatch Omni space summaries describe a separately scoped collaboration namespace, not a SageMaker space or Omni domain; no matching pinned type.",
    },
];
