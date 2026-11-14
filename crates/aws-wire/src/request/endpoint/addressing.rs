use super::super::EncodeError;
use inventory_core::catalog::Target;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointAddressing<'a> {
    pub host_prefix: &'a str,
    pub path_label: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressingStyle {
    Auto,
    Path,
    Virtual,
}
#[derive(Clone, Copy, Debug)]
pub struct S3EndpointConfig<'a> {
    pub endpoint: &'a str,
    pub region: &'a str,
    pub bucket: Option<&'a str>,
    pub style: Option<AddressingStyle>,
    pub dualstack: bool,
    pub accelerate: bool,
    pub operation_style: Option<AddressingStyle>,
    pub explicit_endpoint: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAddressing<'a> {
    pub endpoint: Cow<'a, str>,
    pub addressing: Option<EndpointAddressing<'a>>,
}

pub fn s3_endpoint<'a>(
    config: S3EndpointConfig<'a>,
) -> Result<ResolvedAddressing<'a>, EncodeError> {
    if config.explicit_endpoint {
        return Ok(ResolvedAddressing {
            endpoint: Cow::Borrowed(config.endpoint),
            addressing: None,
        });
    }
    validate_region(config.region)?;
    let style = config
        .operation_style
        .or(config.style)
        .unwrap_or(AddressingStyle::Auto);
    let virtual_host = use_virtual_host(config, style)?;
    let endpoint = s3_url(
        config.endpoint,
        config.region,
        config.dualstack,
        config.accelerate,
    )?;
    let addressing = virtual_host.then_some(EndpointAddressing {
        host_prefix: "{Bucket}.",
        path_label: "Bucket",
    });
    Ok(ResolvedAddressing {
        endpoint,
        addressing,
    })
}
pub(crate) fn resolve<'a>(
    context: super::super::RequestContext<'a>,
    endpoint: &'a str,
) -> Result<Cow<'a, str>, EncodeError> {
    let Some(mapping) = context.addressing else {
        return Ok(Cow::Borrowed(endpoint));
    };
    let prefix = expand_mapping(context, mapping.host_prefix)?;
    super::host::prepend_host(endpoint, &prefix, context.limits.uri_bytes()).map(Cow::Owned)
}

fn expand_mapping(
    context: super::super::RequestContext<'_>,
    template: &str,
) -> Result<String, EncodeError> {
    let mut output = String::new();
    let mut remaining = template;
    while let Some((literal, label)) = remaining.split_once('{') {
        super::append(
            &mut output,
            super::fragment(literal)?,
            context.limits.uri_bytes(),
        )?;
        let (name, rest) = label.split_once('}').ok_or(EncodeError::InvalidEndpoint)?;
        super::append(
            &mut output,
            address_value(context, name)?,
            context.limits.uri_bytes(),
        )?;
        remaining = rest;
    }
    super::append(
        &mut output,
        super::fragment(remaining)?,
        context.limits.uri_bytes(),
    )?;
    Ok(output)
}

fn address_value<'a>(
    context: super::super::RequestContext<'a>,
    name: &str,
) -> Result<&'a str, EncodeError> {
    let Target::Shape(index) = context.operation.input()? else {
        return Err(EncodeError::InvalidEndpoint);
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    for item in shape.members()? {
        let member = item?;
        if member.name()? == name {
            return super::label_value(context.input.get_field(member.id)?, member.id);
        }
    }
    Err(EncodeError::InvalidEndpoint)
}

fn valid_bucket(bucket: &str) -> bool {
    let bytes = bucket.as_bytes();
    (3..=63).contains(&bytes.len())
        && bucket.parse::<std::net::Ipv4Addr>().is_err()
        && bucket.split('.').all(valid_bucket_label)
}

fn valid_bucket_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    !bytes.is_empty()
        && bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn use_virtual_host(
    config: S3EndpointConfig<'_>,
    style: AddressingStyle,
) -> Result<bool, EncodeError> {
    let Some(bucket) = config.bucket else {
        return match style {
            AddressingStyle::Path if config.accelerate => Err(EncodeError::InvalidEndpoint),
            AddressingStyle::Virtual => Err(EncodeError::InvalidEndpoint),
            AddressingStyle::Auto if config.accelerate => Err(EncodeError::InvalidEndpoint),
            AddressingStyle::Path | AddressingStyle::Auto => Ok(false),
        };
    };
    let valid = valid_bucket(bucket);
    let dotted = bucket.contains('.');
    let dotted_https = dotted
        && config
            .endpoint
            .parse::<http::Uri>()
            .map_err(|_| EncodeError::InvalidEndpoint)?
            .scheme_str()
            == Some("https");
    let unsafe_accelerate = config.accelerate && dotted;
    match style {
        AddressingStyle::Path if config.accelerate => Err(EncodeError::InvalidEndpoint),
        AddressingStyle::Path => Ok(false),
        AddressingStyle::Virtual if !valid || dotted_https || unsafe_accelerate => {
            Err(EncodeError::InvalidEndpoint)
        }
        AddressingStyle::Virtual => Ok(true),
        AddressingStyle::Auto if config.accelerate && (!valid || unsafe_accelerate) => {
            Err(EncodeError::InvalidEndpoint)
        }
        AddressingStyle::Auto => Ok(valid && !dotted_https),
    }
}
fn validate_region(region: &str) -> Result<(), EncodeError> {
    let bytes = region.as_bytes();
    if !bytes.is_empty()
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && !region.contains("--")
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        Ok(())
    } else {
        Err(EncodeError::InvalidEndpoint)
    }
}

fn s3_url<'a>(
    endpoint: &'a str,
    region: &str,
    dualstack: bool,
    accelerate: bool,
) -> Result<Cow<'a, str>, EncodeError> {
    let uri = endpoint
        .parse::<http::Uri>()
        .map_err(|_| EncodeError::InvalidEndpoint)?;
    let scheme = uri.scheme_str().ok_or(EncodeError::InvalidEndpoint)?;
    let authority = uri.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let host = if accelerate {
        if dualstack {
            "s3-accelerate.dualstack.amazonaws.com".to_owned()
        } else {
            "s3-accelerate.amazonaws.com".to_owned()
        }
    } else if dualstack {
        format!("s3.dualstack.{region}.amazonaws.com")
    } else {
        format!("s3.{region}.amazonaws.com")
    };
    let port = authority
        .port_u16()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    let path = uri
        .path_and_query()
        .map_or("", http::uri::PathAndQuery::as_str);
    Ok(Cow::Owned(format!("{scheme}://{host}{port}{path}")))
}
