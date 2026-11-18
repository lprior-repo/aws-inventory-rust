use super::{bind, inventory, name, path, token};
use crate::overlay::{
    ExecutionBranch, FixedValue, Identity, NameError, OperationOverlay, OverlayClass, Paginator,
    RequestRouting, Scope, ServiceOverlay,
};
use std::collections::BTreeMap;

/// Literal pinned-model inventory contracts; endpoint bootstrap never emits a resource.
pub(super) fn apply(mut services: Vec<ServiceOverlay>) -> Result<Vec<ServiceOverlay>, NameError> {
    for service in &mut services {
        match service.service.as_str() {
            "timestream-write" => timestream_write(service)?,
            "timestream-query" => timestream_query(service)?,
            "wafv2" => waf(service)?,
            _ => {}
        }
    }
    Ok(services)
}

fn route() -> Result<RequestRouting, NameError> {
    Ok(RequestRouting::Discovered {
        operation: "DescribeEndpoints".to_owned(),
        endpoints_path: path("Endpoints")?,
        address_path: path("Address")?,
        cache_minutes_path: path("CachePeriodInMinutes")?,
        trusted_dns_suffix: ".timestream.{region}.amazonaws.com".to_owned(),
    })
}

fn bootstrap(service: &mut ServiceOverlay) -> Result<(), NameError> {
    service.operations.insert(
        name("DescribeEndpoints")?,
        OperationOverlay {
            class: Some(OverlayClass::Enumerate),
            scope: Some(Scope::Region),
            endpoint_discovery: true,
            ..OperationOverlay::default()
        },
    );
    Ok(())
}

fn discovered(
    service: &mut ServiceOverlay,
    operation: &str,
    mut plan: OperationOverlay,
) -> Result<(), NameError> {
    plan.routing = route()?;
    service.operations.insert(name(operation)?, plan);
    Ok(())
}

fn timestream_write(service: &mut ServiceOverlay) -> Result<(), NameError> {
    bootstrap(service)?;
    for (operation, items, arn) in [
        ("ListDatabases", "Databases", "Arn"),
        ("ListTables", "Tables", "Arn"),
        ("ListBatchLoadTasks", "BatchLoadTasks", "TaskId"),
    ] {
        let identity = if operation == "ListBatchLoadTasks" {
            Identity::Key(vec![path(arn)?])
        } else {
            Identity::Arn(path(arn)?)
        };
        let mut plan = inventory(
            items,
            identity,
            token("NextToken", "NextToken")?,
            Scope::Region,
        )?;
        if operation == "ListTables" {
            bind(&mut plan, "DatabaseName", "ListDatabases", "DatabaseName")?;
        }
        discovered(service, operation, plan)?;
    }
    for (operation, items, parent, bindings, identity) in [
        (
            "DescribeDatabase",
            "Database",
            "ListDatabases",
            &[("DatabaseName", "DatabaseName")][..],
            Identity::Arn(path("Arn")?),
        ),
        (
            "DescribeTable",
            "Table",
            "ListTables",
            &[("DatabaseName", "DatabaseName"), ("TableName", "TableName")][..],
            Identity::Arn(path("Arn")?),
        ),
        (
            "DescribeBatchLoadTask",
            "BatchLoadTaskDescription",
            "ListBatchLoadTasks",
            &[("TaskId", "TaskId")][..],
            Identity::Key(vec![path("TaskId")?]),
        ),
    ] {
        let mut plan = inventory(items, identity, Paginator::None, Scope::Region)?;
        for (input, field) in bindings {
            bind(&mut plan, input, parent, field)?;
        }
        discovered(service, operation, plan)?;
    }
    Ok(())
}

fn timestream_query(service: &mut ServiceOverlay) -> Result<(), NameError> {
    bootstrap(service)?;
    discovered(
        service,
        "ListScheduledQueries",
        inventory(
            "ScheduledQueries",
            Identity::Arn(path("Arn")?),
            token("NextToken", "NextToken")?,
            Scope::Region,
        )?,
    )
}

pub(super) fn branches(bind_scope: bool) -> Result<Vec<ExecutionBranch>, NameError> {
    [
        ("REGIONAL", Scope::Region, None),
        ("CLOUDFRONT", Scope::Account, Some("us-east-1")),
    ]
    .into_iter()
    .map(|(value, scope, region)| {
        Ok(ExecutionBranch {
            discriminator: value.to_owned(),
            fixed: if bind_scope {
                BTreeMap::from([(path("Scope")?, FixedValue::String(value.to_owned()))])
            } else {
                BTreeMap::new()
            },
            scope,
            request_region: region.map(str::to_owned),
            bind: BTreeMap::new(),
            emission: None,
            row_policy: None,
        })
    })
    .collect()
}

fn waf(service: &mut ServiceOverlay) -> Result<(), NameError> {
    for (list, items, get, selected) in [
        ("ListWebACLs", "WebACLs", "GetWebACL", "WebACL"),
        ("ListIPSets", "IPSets", "GetIPSet", "IPSet"),
        (
            "ListRegexPatternSets",
            "RegexPatternSets",
            "GetRegexPatternSet",
            "RegexPatternSet",
        ),
        ("ListRuleGroups", "RuleGroups", "GetRuleGroup", "RuleGroup"),
        (
            "ListManagedRuleSets",
            "ManagedRuleSets",
            "GetManagedRuleSet",
            "ManagedRuleSet",
        ),
    ] {
        let mut enumerator = inventory(
            items,
            Identity::Arn(path("ARN")?),
            token("NextMarker", "NextMarker")?,
            Scope::Region,
        )?;
        enumerator.branches = branches(true)?;
        service.operations.insert(name(list)?, enumerator);
        let mut detail = inventory(
            selected,
            Identity::Arn(path("ARN")?),
            Paginator::None,
            Scope::Region,
        )?;
        bind(&mut detail, "Name", list, "Name")?;
        bind(&mut detail, "Id", list, "Id")?;
        detail.branches = branches(true)?;
        service.operations.insert(name(get)?, detail);
    }
    let mut logging = inventory(
        "LoggingConfigurations",
        Identity::Arn(path("ResourceArn")?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    logging.branches = branches(true)?;
    service
        .operations
        .insert(name("ListLoggingConfigurations")?, logging);
    let mut detail = inventory(
        "LoggingConfiguration",
        Identity::Arn(path("ResourceArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    bind(
        &mut detail,
        "ResourceArn",
        "ListLoggingConfigurations",
        "ResourceArn",
    )?;
    detail.branches = branches(false)?;
    service
        .operations
        .insert(name("GetLoggingConfiguration")?, detail);
    Ok(())
}
