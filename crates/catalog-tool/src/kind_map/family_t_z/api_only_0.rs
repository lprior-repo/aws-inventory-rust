use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping { kind: "AWS::ApiOnly::TextractAdapterVersion", reason: "Immutable adapter model version is a child resource distinct from AWS::Textract::Adapter." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TimestreamInfluxDBParameterGroup", reason: "InfluxDB engine parameter group has no pinned CFN equivalent; it is not an RDS DBParameterGroup." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranscribeCallAnalyticsJob", reason: "Metadata-only asynchronous Transcribe call analytics job has no pinned CFN job type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranscribeLanguageModel", reason: "Custom Transcribe acoustic or language model has no pinned CFN resource type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranscribeMedicalScribeJob", reason: "Metadata-only asynchronous Transcribe medical scribe job has no pinned CFN job type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranscribeMedicalTranscriptionJob", reason: "Metadata-only asynchronous medical transcription job has no pinned CFN job type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranscribeTranscriptionJob", reason: "Metadata-only asynchronous transcription job has no pinned CFN job type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::TranslateTextTranslationJob", reason: "Translate batch translation job metadata is distinct from ParallelData and Terminology; no pinned CFN job type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::VpcLatticeResourceEndpointAssociation", reason: "Resource endpoint association is distinct from a ServiceNetworkVpcAssociation and has no pinned CFN type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::VpcLatticeServiceNetworkVpcEndpointAssociation", reason: "Service network VPC endpoint association is distinct from ServiceNetworkVpcAssociation and has no pinned CFN type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFGeoMatchSet", reason: "Classic global WAF geo match set is absent from pinned CFN; regional and WAFv2 resources have different scope and API." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRateBasedRule", reason: "Classic global WAF rate based rule is absent from pinned CFN; regional and WAFv2 rules are different resources." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRegexMatchSet", reason: "Classic global WAF regex match set is absent from pinned CFN; it is not a regional or WAFv2 pattern set." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRegexPatternSet", reason: "Classic global WAF regex pattern set is absent from pinned CFN; regional and WAFv2 pattern sets are different resources." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRegionalRegexMatchSet", reason: "Classic regional WAF regex match set has no pinned CFN type and is distinct from WAFRegional RegexPatternSet." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRegionalRuleGroup", reason: "Classic regional WAF rule group has no pinned CFN type and is distinct from a WAFv2 rule group." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WAFRuleGroup", reason: "Classic global WAF rule group is absent from pinned CFN; regional and WAFv2 rule groups are different resources." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WorkMailOrganization", reason: "WorkMail organization has a stable OrganizationId but no pinned CFN organization type." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WorkSpacesAccountLink", reason: "WorkSpaces cross-account linkage is neither a directory nor workspace and has no pinned CFN equivalent." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::WorkSpacesApplication", reason: "WorkSpaces application asset is not an AppStream Application and has no pinned CFN equivalent." },
];
