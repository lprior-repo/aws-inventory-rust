// Pinned operation purposes: .local/aws-models/models/sso-oidc/service/2019-06-10/sso-oidc-2019-06-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Full modeled response contains authentication/secret material accessToken, refreshToken. Pinned purpose: Creates and returns access and refresh tokens for clients that are authenticated using client secrets. The access token can be used to fetch short-lived credentials for the assigned AWS accounts or to access application APIs using bearer authentication.. Projection cannot authorize this call.
    ("CreateToken", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material accessToken, refreshToken. Pinned purpose: Creates and returns access and refresh tokens for authorized client applications that are authenticated using any IAM entity, such as a service role or user. These tokens might contain defined scopes that specify permissions such as read:profile or write:data . Through downscoping, you can use the s. Projection cannot authorize this call.
    ("CreateTokenWithIAM", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material clientSecret. Pinned purpose: Registers a public client with IAM Identity Center. This allows clients to perform authorization using the authorization code grant with Proof Key for Code Exchange (PKCE) or the device code grant.. Projection cannot authorize this call.
    ("RegisterClient", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates device authorization by requesting a pair of verification codes from the authorization service.
    ("StartDeviceAuthorization", Decision::Denied(DenyReason::Mutating)),
];
