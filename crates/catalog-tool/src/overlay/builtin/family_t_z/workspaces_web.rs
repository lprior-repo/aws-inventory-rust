use super::super::service;
use super::{defer, deny, extra_plans, workspaces_web_plans_1, workspaces_web_plans_2};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workspaces-web",
        vec![
            ("AssociateBrowserSettings", deny(DenyReason::Mutating)),
            (
                "AssociateDataProtectionSettings",
                deny(DenyReason::Mutating),
            ),
            ("AssociateIpAccessSettings", deny(DenyReason::Mutating)),
            ("AssociateNetworkSettings", deny(DenyReason::Mutating)),
            ("AssociateSessionLogger", deny(DenyReason::Mutating)),
            ("AssociateTrustStore", deny(DenyReason::Mutating)),
            (
                "AssociateUserAccessLoggingSettings",
                deny(DenyReason::Mutating),
            ),
            ("AssociateUserSettings", deny(DenyReason::Mutating)),
            ("CreateBrowserSettings", deny(DenyReason::Mutating)),
            ("CreateDataProtectionSettings", deny(DenyReason::Mutating)),
            ("CreateIdentityProvider", deny(DenyReason::Mutating)),
            ("CreateIpAccessSettings", deny(DenyReason::Mutating)),
            ("CreateNetworkSettings", deny(DenyReason::Mutating)),
            ("CreatePortal", deny(DenyReason::Mutating)),
            ("CreateSessionLogger", deny(DenyReason::Mutating)),
            ("CreateTrustStore", deny(DenyReason::Mutating)),
            (
                "CreateUserAccessLoggingSettings",
                deny(DenyReason::Mutating),
            ),
            ("CreateUserSettings", deny(DenyReason::Mutating)),
            ("DeleteBrowserSettings", deny(DenyReason::Mutating)),
            ("DeleteDataProtectionSettings", deny(DenyReason::Mutating)),
            ("DeleteIdentityProvider", deny(DenyReason::Mutating)),
            ("DeleteIpAccessSettings", deny(DenyReason::Mutating)),
            ("DeleteNetworkSettings", deny(DenyReason::Mutating)),
            ("DeletePortal", deny(DenyReason::Mutating)),
            ("DeleteSessionLogger", deny(DenyReason::Mutating)),
            ("DeleteTrustStore", deny(DenyReason::Mutating)),
            (
                "DeleteUserAccessLoggingSettings",
                deny(DenyReason::Mutating),
            ),
            ("DeleteUserSettings", deny(DenyReason::Mutating)),
            ("DisassociateBrowserSettings", deny(DenyReason::Mutating)),
            (
                "DisassociateDataProtectionSettings",
                deny(DenyReason::Mutating),
            ),
            ("DisassociateIpAccessSettings", deny(DenyReason::Mutating)),
            ("DisassociateNetworkSettings", deny(DenyReason::Mutating)),
            ("DisassociateSessionLogger", deny(DenyReason::Mutating)),
            ("DisassociateTrustStore", deny(DenyReason::Mutating)),
            (
                "DisassociateUserAccessLoggingSettings",
                deny(DenyReason::Mutating),
            ),
            ("DisassociateUserSettings", deny(DenyReason::Mutating)),
            (
                "ExpireSession",
                defer(
                    "Missing executable inventory binding for portalId, sessionId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetBrowserSettings",
                workspaces_web_plans_1::get_browser_settings()?,
            ),
            (
                "GetDataProtectionSettings",
                workspaces_web_plans_1::get_data_protection_settings()?,
            ),
            ("GetIdentityProvider", deny(DenyReason::Secret)),
            (
                "GetIpAccessSettings",
                workspaces_web_plans_1::get_ip_access_settings()?,
            ),
            (
                "GetNetworkSettings",
                workspaces_web_plans_1::get_network_settings()?,
            ),
            ("GetPortal", workspaces_web_plans_1::get_portal()?),
            (
                "GetPortalServiceProviderMetadata",
                extra_plans::get_portal_service_provider_metadata()?,
            ),
            (
                "GetSession",
                defer(
                    "Missing executable inventory binding for portalId, sessionId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetSessionLogger",
                workspaces_web_plans_1::get_session_logger()?,
            ),
            ("GetTrustStore", workspaces_web_plans_1::get_trust_store()?),
            (
                "GetTrustStoreCertificate",
                defer(
                    "Missing executable inventory binding for trustStoreArn, thumbprint: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetUserAccessLoggingSettings",
                workspaces_web_plans_1::get_user_access_logging_settings()?,
            ),
            (
                "GetUserSettings",
                workspaces_web_plans_1::get_user_settings()?,
            ),
            (
                "ListBrowserSettings",
                workspaces_web_plans_1::list_browser_settings()?,
            ),
            (
                "ListDataProtectionSettings",
                workspaces_web_plans_1::list_data_protection_settings()?,
            ),
            (
                "ListIdentityProviders",
                workspaces_web_plans_1::list_identity_providers()?,
            ),
            (
                "ListIpAccessSettings",
                workspaces_web_plans_1::list_ip_access_settings()?,
            ),
            (
                "ListNetworkSettings",
                workspaces_web_plans_2::list_network_settings()?,
            ),
            ("ListPortals", workspaces_web_plans_2::list_portals()?),
            (
                "ListSessionLoggers",
                workspaces_web_plans_2::list_session_loggers()?,
            ),
            (
                "ListSessions",
                defer(
                    "Missing executable inventory binding for portalId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "ListTrustStoreCertificates",
                defer(
                    "Missing executable inventory binding for trustStoreArn: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListTrustStores",
                workspaces_web_plans_2::list_trust_stores()?,
            ),
            (
                "ListUserAccessLoggingSettings",
                workspaces_web_plans_2::list_user_access_logging_settings()?,
            ),
            (
                "ListUserSettings",
                workspaces_web_plans_2::list_user_settings()?,
            ),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateBrowserSettings", deny(DenyReason::Mutating)),
            ("UpdateDataProtectionSettings", deny(DenyReason::Mutating)),
            ("UpdateIdentityProvider", deny(DenyReason::Mutating)),
            ("UpdateIpAccessSettings", deny(DenyReason::Mutating)),
            ("UpdateNetworkSettings", deny(DenyReason::Mutating)),
            ("UpdatePortal", deny(DenyReason::Mutating)),
            ("UpdateSessionLogger", deny(DenyReason::Mutating)),
            ("UpdateTrustStore", deny(DenyReason::Mutating)),
            (
                "UpdateUserAccessLoggingSettings",
                deny(DenyReason::Mutating),
            ),
            ("UpdateUserSettings", deny(DenyReason::Mutating)),
        ],
    )
}
