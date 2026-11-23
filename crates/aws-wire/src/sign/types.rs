use super::Error;
use aws_smithy_runtime_api::client::identity::Identity;
use std::fmt;

#[derive(Clone)]
pub struct Credentials(pub(super) Identity);

impl Credentials {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Region<'a>(pub(super) &'a str);

impl<'a> Region<'a> {
    pub fn new(value: &'a str) -> Result<Self, Error> {
        if !scope_part(value) {
            return Err(Error::Region);
        }
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SigningName<'a>(pub(super) &'a str);

impl<'a> SigningName<'a> {
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

#[derive(Clone, Copy, Debug)]
pub struct Scope<'a> {
    pub region: Region<'a>,

    pub name: SigningName<'a>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum PathMode {
    #[default]
    Standard,

    S3,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadHeader {
    #[default]
    Omit,

    Include,
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Body<'a> {
    Bytes(&'a [u8]),

    Sha256(&'a str),
}

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a http::Method,

    pub uri: &'a str,

    pub headers: &'a http::HeaderMap,

    pub body: Body<'a>,

    pub payload_header: PayloadHeader,
}
