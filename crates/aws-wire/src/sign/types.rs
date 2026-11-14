use super::Error;
use aws_smithy_runtime_api::client::identity::Identity;
use std::fmt;

/// Profile-loader credentials, owned by the AWS library and redacted in `Debug`.
pub struct Credentials(pub(super) Identity);

impl Credentials {
    /// Copies credentials once; no signing-key cache is maintained.
    ///
    /// # Errors
    /// Returns `Error::Credentials` for empty/invalid keys or tokens exceeding 16 KiB.
    /// Keys are at most 128 bytes; the ID is alphanumeric and secrets/tokens visible ASCII.
    pub fn new(id: &str, secret: &str, token: Option<&str>) -> Result<Self, Error> {
        if !visible(id, 128)
            || !id.bytes().all(|b| b.is_ascii_alphanumeric())
            || !visible(secret, 128)
            || token.is_some_and(|t| !visible(t, 16_384))
        {
            return Err(Error::Credentials);
        }
        Ok(Self(
            aws_credential_types::Credentials::new(
                id,
                secret,
                token.map(str::to_owned),
                None,
                "aws-wire",
            )
            .into(),
        ))
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials([REDACTED])")
    }
}

fn visible(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.bytes().all(|b| b.is_ascii_graphic())
}

/// A borrowed signing region, 1..=63 lowercase ASCII letters, digits or hyphens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Region<'a>(pub(super) &'a str);

impl<'a> Region<'a> {
    /// Parses a region without allocating.
    ///
    /// # Errors
    /// Returns `Error::Region` for an invalid character or length.
    pub fn new(value: &'a str) -> Result<Self, Error> {
        if !scope_part(value) {
            return Err(Error::Region);
        }
        Ok(Self(value))
    }
}

/// A borrowed signing name, 1..=63 ASCII letters, digits or hyphens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SigningName<'a>(pub(super) &'a str);

impl<'a> SigningName<'a> {
    /// Parses a signing name (which can differ from the endpoint service name).
    ///
    /// # Errors
    /// Returns `Error::SigningName` for an invalid character or length.
    pub fn new(value: &'a str) -> Result<Self, Error> {
        if value.is_empty()
            || value.len() > 63
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(Error::SigningName);
        }
        Ok(Self(value))
    }
}

fn scope_part(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The region and endpoint-resolved signing name for this request.
#[derive(Clone, Copy, Debug)]
pub struct Scope<'a> {
    /// Region included in the credential scope.
    pub region: Region<'a>,
    /// Service signing name included in the credential scope.
    pub name: SigningName<'a>,
}

/// Canonical path handling; standard mode is the default.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum PathMode {
    /// Normalize dot segments and repeated slashes, and double-encode the path.
    #[default]
    Standard,
    /// Preserve the path and use single encoding, as required for S3 object keys.
    S3,
}

/// Whether to emit and sign `x-amz-content-sha256`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadHeader {
    /// Hash the payload without adding a checksum header.
    #[default]
    Omit,
    /// Add the checksum header (required for S3).
    Include,
}

/// Body input; neither variant copies payload bytes.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Body<'a> {
    /// Borrowed payload, limited to 16 MiB per signing call.
    Bytes(&'a [u8]),
    /// Precomputed SHA-256: exactly 64 lowercase hexadecimal ASCII characters.
    Sha256(&'a str),
}

/// Borrowed, unsigned HTTP request parts; the URI must already be encoded.
#[derive(Debug)]
pub struct Request<'a> {
    /// HTTP method (at most 64 bytes).
    pub method: &'a http::Method,
    /// Absolute HTTP(S) URI, at most 16 KiB, without user information or fragments.
    pub uri: &'a str,
    /// At most 128 headers with values at most 8 KiB each; ASCII values only.
    /// Signing-owned headers must be absent; an optional host must match the URI authority.
    pub headers: &'a http::HeaderMap,
    /// Borrowed bytes or the caller's precomputed checksum.
    pub body: Body<'a>,
    /// Whether to add and sign `x-amz-content-sha256` for this request.
    pub payload_header: PayloadHeader,
}
