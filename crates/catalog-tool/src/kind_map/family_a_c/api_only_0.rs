use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AccountAccessApplication",
        reason: "Account Access Manager application summaries include identity-source and tenant registration state; no exact Account Access Manager application type exists in the pinned CFN set.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AccountAccountInformation",
        reason: "Account identity and lifecycle metadata describes the AWS account itself, not an account-provisioning resource managed by CloudFormation.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AccountGovCloudAccount",
        reason: "The linked GovCloud account ID/state is an account relationship, not provisioning of a new AWS Organizations account.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AccountRegion",
        reason: "Account-specific Region opt-in state is an account/Region registration, not a deployed regional resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ACMAcmeDomainValidation",
        reason: "ACM ACME domain-validation metadata is a per-endpoint domain validation registration, not an ACM Certificate or Route53 RecordSet definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ACMAcmeEndpoint",
        reason: "ACM ACME protocol endpoints are not ACM certificates or private CAs; endpoint ARN identifies a separate ACME access configuration absent from pinned CFN.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ACMAcmeExternalAccountBinding",
        reason: "ACM ACME external-account binding summaries identify role/expiry metadata, not binding key credentials; no exact pinned binding type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AgentRegistryRegistry",
        reason: "Agent Registry control-plane registry definitions govern discovery and record approval, not AgentCore runtime deployment; no matching pinned registry type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ApiGatewayV2PortalProduct",
        reason: "API Gateway portal product publication metadata is distinct from an API, stage, or developer portal; no matching pinned portal-product type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppConfigExperiment",
        reason: "AppConfig experiment-definition summaries are experimentation objects, not applications or configuration profiles; no matching pinned experiment-definition type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppIntegrationsApplicationAssociation",
        reason: "AppIntegrations application-to-client association metadata is distinct from the application definition; no association resource exists in the pinned CFN set.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppIntegrationsDataIntegrationAssociation",
        reason: "AppIntegrations data-integration association metadata identifies a client binding, not the DataIntegration definition; no exact pinned association type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppIntegrationsEventIntegrationAssociation",
        reason: "AppIntegrations event integration client association metadata has its own association ARN and is distinct from EventIntegration; no matching pinned CFN association type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ApplicationCostProfilerReportDefinition",
        reason: "Application Cost Profiler report schedules and destination settings are not CUR report definitions despite similar names; no exact pinned profiler-report type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppRunnerConnection",
        reason: "App Runner source-code provider connection ARN and connection status are not a Service or scaling configuration; no Connection type in the pinned CFN set.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppStreamExportImageTask",
        reason: "An existing AppStream image-export task is a retained export execution, not the AppStream image or resulting EC2 AMI definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppStreamImage",
        reason: "Private AppStream image inventory includes existing images, not ImageBuilder machines or AppBlocks; no exact pinned AppStream image type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AppStreamUsageReportSubscription",
        reason: "An AppStream usage-report subscription describes a reporting schedule and destination bucket, not report content or an AppStream fleet.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ARCZonalShiftManagedResource",
        reason: "ARC managed-resource metadata records zonal-shift readiness of an existing AWS resource; it must not be relabeled as a duplicate load balancer definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ARCZonalShiftZonalShift",
        reason: "An ARC zonal shift is a time-bounded recovery operation identified by zonalShiftId, not the managed load balancer or autoshift configuration.",
    },
];
