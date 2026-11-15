/// Reviewed IAM resource classes that can name a concrete trust graph target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IamPrincipalKind {
    Role,
    User,
    SamlProvider,
    OidcProvider,
}

impl IamPrincipalKind {
    pub fn canonical_kind(self) -> &'static str {
        match self {
            Self::Role => "AWS::IAM::Role",
            Self::User => "AWS::IAM::User",
            Self::SamlProvider => "AWS::IAM::SAMLProvider",
            Self::OidcProvider => "AWS::IAM::OIDCProvider",
        }
    }
}

/// Parse the resource portion of an IAM ARN, not a policy pattern or URL.
/// Account root, service principals, and wildcards intentionally have no resource kind.
pub fn iam_principal_resource_kind(resource: &str) -> Option<IamPrincipalKind> {
    if resource.len() > 2048 || !resource.is_ascii()
        || resource.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || matches!(byte, b'*' | b'?')) {
        return None;
    }
    let (kind, suffix) = resource.split_once('/')?;
    match kind {
        "role" if iam_name_with_path(suffix) => Some(IamPrincipalKind::Role),
        "user" if iam_name_with_path(suffix) => Some(IamPrincipalKind::User),
        "saml-provider" if name(suffix, 128, b"_.-") => Some(IamPrincipalKind::SamlProvider),
        "oidc-provider" if oidc_provider(suffix) => Some(IamPrincipalKind::OidcProvider),
        _ => None,
    }
}

fn name(value: &str, maximum: usize, punctuation: &[u8]) -> bool {
    !value.is_empty() && value.len() <= maximum
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || punctuation.contains(&byte))
}

fn iam_name_with_path(suffix: &str) -> bool {
    let (path, terminal) = suffix.rsplit_once('/').map_or((None, suffix), |(path, terminal)| (Some(path), terminal));
    name(terminal, 64, b"_+=,.@-") && path.is_none_or(|path| {
        // The modeled IAM path includes its leading and trailing slash (max 512 bytes).
        !path.is_empty() && path.len() <= 510 && path.bytes().all(|byte| (b'!'..=b'~').contains(&byte))
    })
}

fn oidc_provider(suffix: &str) -> bool {
    let (host, path) = suffix.split_once('/').map_or((suffix, None), |(host, path)| (host, Some(path)));
    dns_host(host) && path.is_none_or(uri_path)
}

fn dns_host(host: &str) -> bool {
    !host.is_empty() && host.len() <= 253 && host.split('.').all(|label| {
        !label.is_empty() && label.len() <= 63
            && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
            && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn uri_path(path: &str) -> bool {
    // Each percent sign consumes exactly two ASCII hex digits; no decoding or allocation.
    path.bytes().try_fold(0_u8, |remaining, byte| {
        if remaining > 0 {
            return byte.is_ascii_hexdigit().then_some(remaining.saturating_sub(1));
        }
        if byte == b'%' { return Some(2); }
        (byte.is_ascii_alphanumeric() || b"-._~!$&'()+,;=:@/".contains(&byte)).then_some(0)
    }) == Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concrete_iam_names_enforce_terminal_grammar_without_losing_paths() {
        ["role/target", "role/team!#$%&'()+,;=:@[]\\^_`{|}~/name_+=,.@-", "user/a//nested/name", "saml-provider/provider_NAME-1.2"]
            .into_iter().for_each(|resource| assert!(iam_principal_resource_kind(resource).is_some(), "{resource}"));
        ["role/target#", "user/bad%name", "role/", "role//name", "role/team/*", "user/name?", "saml-provider/path/name", "saml-provider/name+", "root", "*"]
            .into_iter().for_each(|resource| assert!(iam_principal_resource_kind(resource).is_none(), "{resource}"));
        assert!(iam_principal_resource_kind(&format!("role/{}", "a".repeat(64))).is_some());
        assert!(iam_principal_resource_kind(&format!("role/{}", "a".repeat(65))).is_none());
        assert!(iam_principal_resource_kind(&format!("saml-provider/{}", "a".repeat(128))).is_some());
        assert!(iam_principal_resource_kind(&format!("saml-provider/{}", "a".repeat(129))).is_none());
    }

    #[test]
    fn oidc_names_require_concrete_dns_host_and_valid_uri_path() {
        ["oidc-provider/accounts.example.com", "oidc-provider/oidc.eks.us-east-1.amazonaws.com/id/ABCD", "oidc-provider/example.com/a%20b/~subject:@"]
            .into_iter().for_each(|resource| assert_eq!(iam_principal_resource_kind(resource), Some(IamPrincipalKind::OidcProvider), "{resource}"));
        ["oidc-provider/example.com#fragment", "oidc-provider/example.com/path#fragment", "oidc-provider/example.com/path?query", "oidc-provider/example.com:443", "oidc-provider/user@example.com", "oidc-provider/-example.com", "oidc-provider/example..com", "oidc-provider/example.com/%", "oidc-provider/example.com/%xy", "oidc-provider/example.com/[bad]", "oidc-provider/https://example.com"]
            .into_iter().for_each(|resource| assert!(iam_principal_resource_kind(resource).is_none(), "{resource}"));
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn reviewed_resource_classes_are_exact_and_terminal_names_are_bounded() {
        [
            ("role/team/reader_+=,.@-", IamPrincipalKind::Role),
            ("user/team/person_+=,.@-", IamPrincipalKind::User),
            ("saml-provider/corporate.IdP_1", IamPrincipalKind::SamlProvider),
            ("oidc-provider/auth.example.com/issuer/v1", IamPrincipalKind::OidcProvider),
        ].into_iter().for_each(|(resource, expected)| {
            assert_eq!(iam_principal_resource_kind(resource), Some(expected), "{resource}");
        });
        for prefix in ["role", "user"] {
            assert!(iam_principal_resource_kind(&format!("{prefix}/{}", "a".repeat(64))).is_some());
            assert!(iam_principal_resource_kind(&format!("{prefix}/{}", "a".repeat(65))).is_none());
            assert!(iam_principal_resource_kind(&format!("{prefix}/{}/name", "a".repeat(510))).is_some());
            assert!(iam_principal_resource_kind(&format!("{prefix}/{}/name", "a".repeat(511))).is_none());
            for terminal in ["name#fragment", "bad%name", "bad:name", "bad/name?", "é", "bad name", "name\u{7f}"] {
                assert!(iam_principal_resource_kind(&format!("{prefix}/{terminal}")).is_none(), "{prefix}/{terminal}");
            }
        }
    }

    #[test]
    fn oidc_dns_and_escape_boundaries_reject_nonconcrete_issuers() {
        let valid_label = format!("oidc-provider/{}.example.com/issuer%2Fid", "a".repeat(63));
        assert_eq!(iam_principal_resource_kind(&valid_label), Some(IamPrincipalKind::OidcProvider));
        assert!(iam_principal_resource_kind(&format!("oidc-provider/{}.example.com", "a".repeat(64))).is_none());
        let host = format!("{}.{}.{}.{}", "a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(61));
        assert_eq!(host.len(), 253);
        assert_eq!(iam_principal_resource_kind(&format!("oidc-provider/{host}")), Some(IamPrincipalKind::OidcProvider));
        assert!(iam_principal_resource_kind(&format!("oidc-provider/{host}x")).is_none());
        for issuer in ["example.com/user@tenant:scope", "example.com/%2f", "example.com/"] {
            assert_eq!(iam_principal_resource_kind(&format!("oidc-provider/{issuer}")), Some(IamPrincipalKind::OidcProvider));
        }
        for issuer in ["user:pass@example.com", "example.com:443/path", "[::1]/path", "bad_host.example", "example.com/a?b", "example.com/a#b", "example.com/a%2", "example.com/a%GG", "example.com/path*", "example.com/path?"] {
            assert!(iam_principal_resource_kind(&format!("oidc-provider/{issuer}")).is_none(), "{issuer}");
        }
    }
}
