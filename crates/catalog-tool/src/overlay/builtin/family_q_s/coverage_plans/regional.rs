use super::*;
use crate::overlay::{
    IdentityBranch, IdentityChoice, IdentityLeaf, IdentityPredicate, RequestRouting,
};

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    match service.service.as_str() {
        "swf" => swf(service),
        "rbin" => {
            let mut plan = arn("Rules", "RuleArn", Some(("NextToken", "NextToken")))?;
            branches(
                &mut plan,
                "ResourceType",
                &["EBS_VOLUME", "EBS_SNAPSHOT", "EC2_IMAGE"],
            )?;
            put(service, "ListRules", plan)
        }
        "ssm" => ssm(service),
        "ram" => {
            let mut plan = arn("permissions", "arn", Some(("nextToken", "nextToken")))?;
            plan.fixed = [
                (
                    path("resourceOwner")?,
                    FixedValue::String("SELF".to_owned()),
                ),
                (
                    path("permissionType")?,
                    FixedValue::String("CUSTOMER_MANAGED".to_owned()),
                ),
            ]
            .into();
            plan.tags = Some(path("tags")?);
            put(service, "ListPermissions", plan)
        }
        "route53globalresolver" => globalresolver(service),
        "route53-recovery-control-config" => safety_rules(service),
        _ => Ok(()),
    }
}

fn swf(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut domains = arn(
        "domainInfos",
        "arn",
        Some(("nextPageToken", "nextPageToken")),
    )?;
    domains.queryable.push(path("name")?);
    branches(
        &mut domains,
        "registrationStatus",
        &["REGISTERED", "DEPRECATED"],
    )?;
    put(service, "ListDomains", domains)?;
    [
        ("ListActivityTypes", "activityType"),
        ("ListWorkflowTypes", "workflowType"),
    ]
    .into_iter()
    .try_for_each(|(operation, member)| {
        let mut plan = qualified(
            "typeInfos",
            &["domain"],
            &[&format!("{member}.name"), &format!("{member}.version")],
            Some(("nextPageToken", "nextPageToken")),
        )?;
        parent(&mut plan, "domain", "ListDomains", "name")?;
        branches(
            &mut plan,
            "registrationStatus",
            &["REGISTERED", "DEPRECATED"],
        )?;
        put(service, operation, plan)
    })
}

fn ssm(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut documents = qualified(
        "DocumentIdentifiers",
        &[],
        &["Name"],
        Some(("NextToken", "NextToken")),
    )?;
    documents.fixed.insert(
        path("Filters")?,
        FixedValue::List(vec![FixedValue::Structure(
            [
                ("Key".to_owned(), FixedValue::String("Owner".to_owned())),
                (
                    "Values".to_owned(),
                    FixedValue::Strings(vec!["Self".to_owned()]),
                ),
            ]
            .into(),
        )]),
    );
    documents.tags = Some(path("Tags")?);
    put(service, "ListDocuments", documents)?;
    let mut targets = qualified(
        "Targets",
        &["WindowId"],
        &["WindowTargetId"],
        Some(("NextToken", "NextToken")),
    )?;
    parent(
        &mut targets,
        "WindowId",
        "DescribeMaintenanceWindows",
        "WindowId",
    )?;
    put(service, "DescribeMaintenanceWindowTargets", targets)
}

fn globalresolver(service: &mut ServiceOverlay) -> Result<(), NameError> {
    [
        ("ListGlobalResolvers", "globalResolvers", Some("arn"), None),
        ("ListAccessSources", "accessSources", Some("arn"), None),
        (
            "ListFirewallDomainLists",
            "firewallDomainLists",
            Some("arn"),
            None,
        ),
        (
            "ListHostedZoneAssociations",
            "hostedZoneAssociations",
            None,
            None,
        ),
        (
            "ListDNSViews",
            "dnsViews",
            Some("arn"),
            Some(("globalResolverId", "ListGlobalResolvers")),
        ),
        (
            "ListFirewallRules",
            "firewallRules",
            None,
            Some(("dnsViewId", "ListDNSViews")),
        ),
    ]
    .into_iter()
    .try_for_each(|(operation, items, field, source)| {
        let mut plan = match field {
            Some(field) => arn(items, field, Some(("nextToken", "nextToken")))?,
            None => qualified(items, &[], &["id"], Some(("nextToken", "nextToken")))?,
        };
        if let Some((input, source)) = source {
            parent(&mut plan, input, source, "id")?;
        }
        plan.queryable.push(path("id")?);
        plan.scope = Some(Scope::Account);
        plan.routing = RequestRouting::FixedRegion {
            region: "us-east-2".to_owned(),
        };
        put(service, operation, plan)
    })
}

fn safety_rules(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let arms = ["ASSERTION", "GATING"]
        .into_iter()
        .map(|arm| {
            let arn = path(&format!("{arm}.SafetyRuleArn"))?;
            IdentityBranch::new(
                IdentityPredicate::Present(arn.clone()),
                IdentityLeaf::Arn(arn),
            )
        })
        .collect::<Result<_, _>>()?;
    let mut plan = list(
        "SafetyRules",
        Identity::Choice(IdentityChoice::Discriminated(arms)),
        Some(("NextToken", "NextToken")),
    )?;
    parent(
        &mut plan,
        "ControlPanelArn",
        "ListControlPanels",
        "ControlPanelArn",
    )?;
    plan.scope = Some(Scope::Account);
    plan.queryable = ["ASSERTION.SafetyRuleArn", "GATING.SafetyRuleArn"]
        .into_iter()
        .map(path)
        .collect::<Result<_, _>>()?;
    put(service, "ListSafetyRules", plan)
}
