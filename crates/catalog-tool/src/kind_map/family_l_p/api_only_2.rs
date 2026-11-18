use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PartnerSolution",
        reason: "PartnerSolution is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PinpointRecommenderConfiguration",
        reason: "PinpointRecommenderConfiguration is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PinpointSmsVoiceConfigurationSet",
        reason: "PinpointSmsVoiceConfigurationSet is the legacy pinpoint-sms-voice (2018-09-05) name-keyed configuration set; it is absent from the pinned 1881 CloudFormation schemas and must not share the ARN-keyed AWS::SMSVOICE::ConfigurationSet identity domain.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PollyLexicon",
        reason: "PollyLexicon is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ProtonComponent",
        reason: "ProtonComponent is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ProtonEnvironment",
        reason: "ProtonEnvironment is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ProtonRepository",
        reason: "ProtonRepository is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ProtonService",
        reason: "ProtonService is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::ProtonServiceInstance",
        reason: "ProtonServiceInstance is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::UserSubscriptionIdentityProvider",
        reason: "UserSubscriptionIdentityProvider is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::UserSubscriptionInstance",
        reason: "UserSubscriptionInstance is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::UserSubscriptionLicenseServerEndpoint",
        reason: "UserSubscriptionLicenseServerEndpoint is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
];
