use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteDBClusterParameterGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBClusterSnapshot",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBInstance",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBInstanceAutomatedBackup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBParameterGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBProxy",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBProxyEndpoint",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBSecurityGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBShardGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBSnapshot",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteDBSubnetGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteEventSubscription",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteGlobalCluster",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteIntegration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteOptionGroup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteTenantDatabase",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeregisterDBProxyTargets",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DescribeAccountAttributes",
            OperationOverlay {
                class: Some(OverlayClass::Enumerate),
                scope: Some(Scope::Region),
                items: Some(ItemSelector::parse("AccountQuotas")?),
                paginator: Some(Paginator::None),
                identity: Some(Identity::Key(vec![path("AccountQuotaName")?])),
                ..empty()
            },
        ),
        (
            "DescribeBlueGreenDeployments",
            OperationOverlay {
                class: Some(OverlayClass::Enumerate),
                scope: Some(Scope::Region),
                items: Some(ItemSelector::parse("BlueGreenDeployments")?),
                page_size: Some(100),
                tags: Some(path("TagList")?),
                paginator: Some(Paginator::Token(TokenPair {
                    input: path("Marker")?,
                    output: path("Marker")?,
                    optional: false,
                })),
                identity: Some(Identity::Key(vec![path("BlueGreenDeploymentIdentifier")?])),
                ..empty()
            },
        ),
        (
            "DescribeCertificates",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
                ..empty()
            },
        ),
    ])
}
