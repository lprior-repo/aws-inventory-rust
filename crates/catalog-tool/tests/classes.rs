use catalog_tool::classes::{Class, DenyList, DenyReason, OperationFacts, classify};

const MUTATING: &[&str] = &[
    "CreateQueue",
    "DeleteBucket",
    "PutObject",
    "StartInstances",
    "TagResource",
    "Invoke",
    "Publish",
    "listQueues",
    "describeInstances",
    "getObject",
    "",
    "🚀ListThings",
];
const UNCLASSIFIED: &[&str] = &[
    "GetBucketPolicy",
    "BatchGetItem",
    "SearchResources",
    "LookupEvents",
    "SelectResourceConfig",
    "ScanUnknown",
    "QueryUnknown",
];
const MUST_DENY: &[(&str, DenyReason)] = &[
    ("GetId", DenyReason::Mutating),
    ("ReceiveMessage", DenyReason::Mutating),
    ("GetCostAndUsage", DenyReason::Paid),
    ("GetSecretValue", DenyReason::Secret),
    ("GetFunction", DenyReason::Content),
    ("GetObject", DenyReason::Content),
    ("Scan", DenyReason::Content),
    ("Query", DenyReason::Content),
    ("GetParameter", DenyReason::Secret),
    ("GetPasswordData", DenyReason::Secret),
    ("GetLogEvents", DenyReason::Content),
    ("GetMetricData", DenyReason::Paid),
    ("GetAuthorizationToken", DenyReason::Secret),
    ("GetSessionToken", DenyReason::Secret),
    ("GetRecords", DenyReason::Content),
];

fn facts(name: &str, required: bool, readonly: bool) -> OperationFacts<'_> {
    OperationFacts {
        service: "test",
        name,
        has_required_input: required,
        readonly,
    }
}

#[test]
fn list_describe_candidates_depend_on_required_members_not_readonly()
-> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::builtin();
    ["ListQueues", "DescribeInstances", "ListObjectsV2"]
        .iter()
        .for_each(|name| {
            [false, true].iter().for_each(|readonly| {
                assert_eq!(
                    classify(&facts(name, false, *readonly), &deny),
                    Class::Enumerate
                );
                assert_eq!(
                    classify(&facts(name, true, *readonly), &deny),
                    Class::Expand
                );
            });
        });
    Ok(())
}

#[test]
fn literal_list_describe_prefixes_allow_empty_and_lowercase_suffixes()
-> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::parse("")?;
    [
        "List",
        "Describe",
        "Listen",
        "Listqueues",
        "Describeinstances",
    ]
    .iter()
    .for_each(|name| {
        assert_eq!(
            classify(&facts(name, false, false), &deny),
            Class::Enumerate
        );
        assert_eq!(classify(&facts(name, true, true), &deny), Class::Expand);
    });
    Ok(())
}

#[test]
fn nonread_names_are_denied_mutating_even_when_readonly() -> Result<(), Box<dyn std::error::Error>>
{
    let deny = DenyList::builtin();
    MUTATING.iter().for_each(|name| {
        assert_eq!(
            classify(&facts(name, false, true), &deny),
            Class::Denied(DenyReason::Mutating)
        );
    });
    Ok(())
}

#[test]
fn every_seed_denial_keeps_its_hand_reviewed_reason() -> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::builtin();
    assert_eq!(deny.len(), 85);
    MUST_DENY.iter().for_each(|(name, reason)| {
        assert_eq!(
            classify(&facts(name, false, true), &deny),
            Class::Denied(*reason)
        );
        assert_eq!(
            classify(&facts(name, true, false), &deny),
            Class::Denied(*reason)
        );
    });
    Ok(())
}

#[test]
fn other_read_prefixes_stay_unclassified_even_when_input_free_and_readonly()
-> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::builtin();
    UNCLASSIFIED.iter().for_each(|name| {
        assert_eq!(
            classify(&facts(name, false, true), &deny),
            Class::Unclassified
        );
        assert_eq!(
            classify(&facts(name, true, false), &deny),
            Class::Unclassified
        );
    });
    Ok(())
}

#[test]
fn list_denial_wins_over_both_allow_rules() -> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::parse("ListForbidden\treference-data\n")?;
    assert_eq!(
        classify(&facts("ListForbidden", false, true), &deny),
        Class::Denied(DenyReason::ReferenceData)
    );
    assert_eq!(
        classify(&facts("ListForbidden", true, true), &deny),
        Class::Denied(DenyReason::ReferenceData)
    );
    Ok(())
}
