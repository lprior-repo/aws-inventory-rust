use super::super::{name, path};
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    RequestRouting, Scope, ServiceOverlay, TokenPair,
};

pub(super) fn apply(mut service: ServiceOverlay) -> Result<ServiceOverlay, NameError> {
    match service.service.as_str() {
        "route-53-domains" => {
            let plan = plan(
                "Domains",
                Identity::Key(vec![path("DomainName")?]),
                Some(("Marker", "NextPageMarker")),
                "us-east-1",
            )?;
            service.operations.insert(name("ListDomains")?, plan);
        }
        "route53-recovery-readiness" => readiness(&mut service)?,
        "s3-control" => s3(&mut service)?,
        _ => {}
    }
    Ok(service)
}

fn plan(
    items: &str,
    identity: Identity,
    pagination: Option<(&str, &str)>,
    region: &str,
) -> Result<OperationOverlay, NameError> {
    Ok(OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse(items)?),
        identity: Some(identity),
        paginator: Some(match pagination {
            Some((input, output)) => Paginator::Token(TokenPair {
                input: path(input)?,
                output: path(output)?,
                optional: false,
            }),
            None => Paginator::None,
        }),
        routing: RequestRouting::FixedRegion {
            region: region.to_owned(),
        },
        ..OperationOverlay::default()
    })
}

fn readiness(service: &mut ServiceOverlay) -> Result<(), NameError> {
    for (list, items, arn, get, request, field) in [
        (
            "ListCells",
            "Cells",
            "CellArn",
            "GetCell",
            "CellName",
            "CellName",
        ),
        (
            "ListReadinessChecks",
            "ReadinessChecks",
            "ReadinessCheckArn",
            "GetReadinessCheck",
            "ReadinessCheckName",
            "ReadinessCheckName",
        ),
        (
            "ListRecoveryGroups",
            "RecoveryGroups",
            "RecoveryGroupArn",
            "GetRecoveryGroup",
            "RecoveryGroupName",
            "RecoveryGroupName",
        ),
        (
            "ListResourceSets",
            "ResourceSets",
            "ResourceSetArn",
            "GetResourceSet",
            "ResourceSetName",
            "ResourceSetName",
        ),
    ] {
        let mut enumerator = plan(
            items,
            Identity::Arn(path(arn)?),
            Some(("NextToken", "NextToken")),
            "us-west-2",
        )?;
        enumerator.tags = Some(path("Tags")?);
        service.operations.insert(name(list)?, enumerator);
        let mut detail = plan("$", Identity::Arn(path(arn)?), None, "us-west-2")?;
        detail.class = Some(OverlayClass::Expand);
        detail.bind.insert(
            path(request)?,
            Binding::From {
                operation: name(list)?,
                path: path(field)?,
            },
        );
        detail.tags = Some(path("Tags")?);
        service.operations.insert(name(get)?, detail);
    }
    Ok(())
}

fn s3(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut mrap = plan(
        "AccessPoints",
        Identity::Template("arn:{partition}:s3::{account}:accesspoint/{Alias}".to_owned()),
        Some(("NextToken", "NextToken")),
        "us-west-2",
    )?;
    mrap.bind.insert(path("AccountId")?, Binding::Account);
    service
        .operations
        .insert(name("ListMultiRegionAccessPoints")?, mrap);
    let mut directory = plan(
        "AccessPointList",
        Identity::Arn(path("AccessPointArn")?),
        Some(("NextToken", "NextToken")),
        "us-east-1",
    )?;
    directory.scope = Some(Scope::Region);
    directory.routing = RequestRouting::S3ExpressControl;
    directory.bind.insert(path("AccountId")?, Binding::Account);
    service
        .operations
        .insert(name("ListAccessPointsForDirectoryBuckets")?, directory);
    let mut regional = plan(
        "RegionalBucketList",
        Identity::Arn(path("BucketArn")?),
        Some(("NextToken", "NextToken")),
        "us-east-1",
    )?;
    regional.scope = Some(Scope::Region);
    regional.routing = RequestRouting::S3Outposts;
    regional.bind.insert(path("AccountId")?, Binding::Account);
    regional.bind.insert(
        path("OutpostId")?,
        Binding::FromService {
            service: "outposts".to_owned(),
            operation: name("ListOutposts")?,
            path: path("OutpostId")?,
        },
    );
    service
        .operations
        .insert(name("ListRegionalBuckets")?, regional);
    Ok(())
}
