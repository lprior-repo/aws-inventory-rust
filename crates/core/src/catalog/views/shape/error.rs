use super::super::super::{CatalogError, CatalogKey};
use crate::record::{MapRef, ValueRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Client,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModeledError<'a> {
    pub kind: ErrorKind,
    pub code: Option<&'a str>,
    pub status: Option<u16>,
    pub http_status: Option<u16>,
    pub aws_query_status: Option<u16>,
    pub retryable: bool,
    pub throttling: bool,
}

pub(super) fn read(map: MapRef<'_>) -> Result<Option<ModeledError<'_>>, CatalogError> {
    let error_kind = optional_text(map, CatalogKey::ErrorKind)?;
    let http_status = optional_status(map, CatalogKey::HttpError)?;
    let aws_query_status = optional_status(map, CatalogKey::AwsQueryStatus)?;
    let code = optional_text(map, CatalogKey::AwsQueryError)?;
    if error_kind.is_none() && http_status.is_none() && aws_query_status.is_none() && code.is_none()
    {
        return Ok(None);
    }
    let kind = kind(error_kind, http_status)?;
    Ok(Some(ModeledError {
        kind,
        code,
        status: http_status.or(aws_query_status),
        http_status,
        aws_query_status,
        retryable: optional_flag(map, CatalogKey::ErrorRetryable)?,
        throttling: optional_flag(map, CatalogKey::ErrorThrottling)?,
    }))
}

fn kind(error_kind: Option<&str>, http_status: Option<u16>) -> Result<ErrorKind, CatalogError> {
    match error_kind {
        Some("client") => Ok(ErrorKind::Client),
        Some("server") => Ok(ErrorKind::Server),
        None if http_status.is_some_and(|status| status < 500) => Ok(ErrorKind::Client),
        None if http_status.is_some() => Ok(ErrorKind::Server),
        _ => Err(CatalogError::Reference),
    }
}

fn optional_text(map: MapRef<'_>, key: CatalogKey) -> Result<Option<&str>, CatalogError> {
    match map.get_field(key.field()?)? {
        None => Ok(None),
        Some(ValueRef::Text(value)) => Ok(Some(value)),
        _ => Err(CatalogError::Field(key)),
    }
}

fn optional_status(map: MapRef<'_>, key: CatalogKey) -> Result<Option<u16>, CatalogError> {
    let value = match map.get_field(key.field()?)? {
        None => return Ok(None),
        Some(ValueRef::Int(value)) => value,
        _ => return Err(CatalogError::Field(key)),
    };
    let value = u16::try_from(value).map_err(|_| CatalogError::Field(key))?;
    if (100..=599).contains(&value) {
        Ok(Some(value))
    } else {
        Err(CatalogError::Field(key))
    }
}

fn optional_flag(map: MapRef<'_>, key: CatalogKey) -> Result<bool, CatalogError> {
    match map.get_field(key.field()?)? {
        None => Ok(false),
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(CatalogError::Field(key)),
    }
}
