//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/healthlake/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{ContextSource, ExecutionBranch, FixedValue, Identity, IdentityField, MemberPath, NameError, OperationName, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListFHIRDatastores", policy::query(policy::enumerate("DatastorePropertiesList", policy::arn("DatastoreArn")?, Some(("NextToken", "NextToken")))?, &["DatastoreArn"])?),
        ("DescribeFHIRDatastore", policy::query(policy::bind(policy::enumerate("DatastoreProperties", policy::arn("DatastoreArn")?, None)?, "DatastoreId", "ListFHIRDatastores", "DatastoreId")?, &["DatastoreArn"])?),
        ("ListDataTransformationProfiles", profiles()?),
        ("ListDataTransformationProfileVersions", policy::query(policy::bind(
            policy::enumerate("Items", Identity::CorrelatedKey(vec![
                IdentityField::Context { operation: OperationName::new("ListDataTransformationProfiles")?, source: ContextSource::Request, path: MemberPath::new("SourceFormat")? },
                IdentityField::Request(MemberPath::new("ProfileId")?),
                IdentityField::SelectedInteger { path: MemberPath::new("Version")?, minimum: 0, maximum: 99 },
            ]), Some(("NextToken", "NextToken")))?,
            "ProfileId", "ListDataTransformationProfiles", "ProfileId")?,
            &["SourceFormat", "ProfileId", "Version"])?),
    ];
    policy::finish(
        "healthlake", approved, DEFERRED,
        "Datastore infrastructure and transformation-profile/version summaries only, never FHIR records or transformation definitions. Both CCDA and CSV roots are required. Job discovery and sensitive-output detail remain unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

fn profiles() -> Result<crate::overlay::OperationOverlay, NameError> {
    let mut value = policy::query(policy::enumerate("Items",
        Identity::Key(vec![MemberPath::new("SourceFormat")?, MemberPath::new("ProfileId")?]),
        Some(("NextToken", "NextToken")))?, &["SourceFormat", "ProfileId", "Version"])?;
    value.branches = ["CCDA", "CSV"].into_iter().map(|format| {
        Ok(ExecutionBranch {
            discriminator: format.to_owned(),
            fixed: [(MemberPath::new("SourceFormat")?, FixedValue::String(format.to_owned()))].into_iter().collect(),
            scope: Scope::Region,
            request_region: None,
            bind: Default::default(),
            emission: None,
            row_policy: None,
        })
    }).collect::<Result<_, NameError>>()?;
    Ok(value)
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
// The pinned file also declares TransformData, ValidateSource, and duplicate tag operations in an
// unbound transformation namespace; they are outside the HealthLake service closure and not reviewed.
const DEFERRED: &[&str] = &[
    "DescribeDataTransformationJob", "DescribeFHIRExportJob", "DescribeFHIRImportJob",
    "ListDataTransformationJobs",
    "ListFHIRExportJobs", "ListFHIRImportJobs", "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateDataTransformationProfile", "CreateFHIRDatastore", "DeleteDataTransformationProfile",
    "DeleteFHIRDatastore", "PublishDataTransformationProfile", "RestoreFHIRDatastore",
    "StartDataTransformationJob", "StartFHIRExportJob", "StartFHIRImportJob", "TagResource",
    "UntagResource", "UpdateDataTransformationProfile", "UpdateFHIRDatastore",
    "UpdateProfileWithAgent",
];

const CONTENT: &[&str] = &[
    "GetDataTransformationProfile",
];
