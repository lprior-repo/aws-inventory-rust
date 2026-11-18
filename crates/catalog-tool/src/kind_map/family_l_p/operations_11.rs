use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "pinpoint-sms-voice-v2",
        operation: "DescribeVerifiedDestinationNumbers",
        kind: "AWS::SMSVOICE::VerifiedDestinationNumber",
    },
    OperationMapping {
        service: "pinpoint-sms-voice",
        operation: "ListConfigurationSets",
        kind: "AWS::ApiOnly::PinpointSmsVoiceConfigurationSet",
    },
    OperationMapping {
        service: "pinpoint",
        operation: "GetApps",
        kind: "AWS::Pinpoint::App",
    },
    OperationMapping {
        service: "pinpoint",
        operation: "GetRecommenderConfigurations",
        kind: "AWS::ApiOnly::PinpointRecommenderConfiguration",
    },
    OperationMapping {
        service: "pipes",
        operation: "ListPipes",
        kind: "AWS::Pipes::Pipe",
    },
    OperationMapping {
        service: "proton",
        operation: "ListComponents",
        kind: "AWS::ApiOnly::ProtonComponent",
    },
    OperationMapping {
        service: "proton",
        operation: "ListEnvironmentTemplates",
        kind: "AWS::Proton::EnvironmentTemplate",
    },
    OperationMapping {
        service: "proton",
        operation: "ListEnvironments",
        kind: "AWS::ApiOnly::ProtonEnvironment",
    },
    OperationMapping {
        service: "proton",
        operation: "ListRepositories",
        kind: "AWS::ApiOnly::ProtonRepository",
    },
    OperationMapping {
        service: "proton",
        operation: "ListServiceInstances",
        kind: "AWS::ApiOnly::ProtonServiceInstance",
    },
    OperationMapping {
        service: "proton",
        operation: "ListServiceTemplates",
        kind: "AWS::Proton::ServiceTemplate",
    },
    OperationMapping {
        service: "proton",
        operation: "ListServices",
        kind: "AWS::ApiOnly::ProtonService",
    },
    OperationMapping {
        service: "polly",
        operation: "ListLexicons",
        kind: "AWS::ApiOnly::PollyLexicon",
    },
    OperationMapping {
        service: "marketplace-agreement",
        operation: "SearchAgreements",
        kind: "AWS::ApiOnly::MarketplaceAgreement",
    },
    OperationMapping {
        service: "partnercentral-revenue-measurement",
        operation: "ListRevenueAttributions",
        kind: "AWS::ApiOnly::PartnerRevenueAttribution",
    },
    OperationMapping {
        service: "partnercentral-revenue-measurement",
        operation: "ListMarketplaceRevenueShares",
        kind: "AWS::ApiOnly::PartnerMarketplaceRevenueShare",
    },
    OperationMapping {
        service: "partnercentral-selling",
        operation: "ListSolutions",
        kind: "AWS::ApiOnly::PartnerSolution",
    },
    OperationMapping {
        service: "partnercentral-selling",
        operation: "ListEngagements",
        kind: "AWS::ApiOnly::PartnerEngagement",
    },
    OperationMapping {
        service: "pricing-plan-manager",
        operation: "ListSubscriptions",
        kind: "AWS::ApiOnly::FlatRatePricingSubscription",
    },
];
