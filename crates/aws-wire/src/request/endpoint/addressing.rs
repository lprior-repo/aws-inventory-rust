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
        config.bucket,
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

fn is_directory_bucket(bucket: &str) -> bool {
    bucket.ends_with("--x-s3") && bucket.split("--").count() == 3
}

fn extract_zone(bucket: &str) -> Option<&str> {
    let mut split = bucket.split("--");
    let base = split.next()?;
    let zone = split.next()?;
    let suffix = split.next()?;
    if suffix != "x-s3" || split.next().is_some() {
        return None;
    }
    if base.is_empty() || zone.is_empty() {
        return None;
    }
    Some(zone)
}

fn valid_zone_id(zone: &str) -> bool {
    let mut parts = zone.split('-');
    let prefix = parts.next().unwrap_or("");
    let az_part = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false;
    }
    let Some(num) = az_part.strip_prefix("az") else {
        return false;
    };
    if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    if num.len() > 1 && num.starts_with('0') {
        return false;
    }
    if prefix.len() < 3 || prefix.len() > 10 {
        return false;
    }
    if !prefix
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    {
        return false;
    }
    if !prefix
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_lowercase())
    {
        return false;
    }
    if !prefix.bytes().last().is_some_and(|b| b.is_ascii_digit()) {
        return false;
    }
    prefix.bytes().filter(|b| b.is_ascii_lowercase()).count() >= 2
}

fn valid_directory_bucket(bucket: &str) -> bool {
    if bucket.len() < 3 || bucket.len() > 63 {
        return false;
    }
    if bucket.contains('.') || bucket.contains('_') {
        return false;
    }
    let zone = match extract_zone(bucket) {
        Some(z) if valid_zone_id(z) => z,
        _ => return false,
    };
    let _ = zone;
    let base = bucket.split("--").next().unwrap_or("");
    if base.is_empty() || base.len() < 3 {
        return false;
    }
    if base.starts_with("xn--") {
        return false;
    }
    if base.parse::<std::net::Ipv4Addr>().is_ok() {
        return false;
    }
    if !base
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return false;
    }
    if base.starts_with('-') || base.ends_with('-') {
        return false;
    }
    if base.contains("--") {
        return false;
    }
    bucket
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
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
    if is_directory_bucket(bucket) {
        match style {
            AddressingStyle::Path => return Err(EncodeError::InvalidEndpoint),
            AddressingStyle::Virtual | AddressingStyle::Auto => {
                if config.accelerate {
                    return Err(EncodeError::InvalidEndpoint);
                }
                if !valid_directory_bucket(bucket) {
                    return Err(EncodeError::InvalidEndpoint);
                }
                return Ok(true);
            }
        }
    }
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
    bucket: Option<&'a str>,
    dualstack: bool,
    accelerate: bool,
) -> Result<Cow<'a, str>, EncodeError> {
    let uri = endpoint
        .parse::<http::Uri>()
        .map_err(|_| EncodeError::InvalidEndpoint)?;
    let scheme = uri.scheme_str().ok_or(EncodeError::InvalidEndpoint)?;
    let authority = uri.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let host = if let Some(b) = bucket {
        if is_directory_bucket(b) {
            if accelerate {
                return Err(EncodeError::InvalidEndpoint);
            }
            let zone = extract_zone(b).ok_or(EncodeError::InvalidEndpoint)?;
            if !valid_zone_id(zone) {
                return Err(EncodeError::InvalidEndpoint);
            }
            if dualstack {
                format!("s3express-{zone}.dualstack.{region}.amazonaws.com")
            } else {
                format!("s3express-{zone}.{region}.amazonaws.com")
            }
        } else if accelerate {
            if dualstack {
                "s3-accelerate.dualstack.amazonaws.com".to_owned()
            } else {
                "s3-accelerate.amazonaws.com".to_owned()
            }
        } else if dualstack {
            format!("s3.dualstack.{region}.amazonaws.com")
        } else {
            format!("s3.{region}.amazonaws.com")
        }
    } else if accelerate {
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

#[cfg(test)]
mod tests {
    use super::{AddressingStyle, S3EndpointConfig, s3_endpoint};

    fn cfg<'a>(
        bucket: Option<&'a str>,
        style: Option<AddressingStyle>,
        operation_style: Option<AddressingStyle>,
        dualstack: bool,
        accelerate: bool,
    ) -> S3EndpointConfig<'a> {
        S3EndpointConfig {
            endpoint: "https://s3.us-west-2.amazonaws.com",
            region: "us-west-2",
            bucket,
            style,
            dualstack,
            accelerate,
            operation_style,
            explicit_endpoint: false,
        }
    }

    #[test]
    fn s3_regular_virtual_hosted_produces_regional_host() -> Result<(), Box<dyn std::error::Error>>
    {
        let resolved = s3_endpoint(cfg(
            Some("mybucket"),
            Some(AddressingStyle::Virtual),
            None,
            false,
            false,
        ))?;
        assert_eq!(
            resolved.endpoint.as_ref(),
            "https://s3.us-west-2.amazonaws.com/"
        );
        let addressing = resolved.addressing.ok_or("virtual hosted addressing")?;
        assert_eq!(addressing.host_prefix, "{Bucket}.");
        assert_eq!(addressing.path_label, "Bucket");
        Ok(())
    }

    #[test]
    fn s3_regular_path_produces_no_addressing() -> Result<(), Box<dyn std::error::Error>> {
        let resolved = s3_endpoint(cfg(
            Some("mybucket"),
            Some(AddressingStyle::Path),
            None,
            false,
            false,
        ))?;
        assert!(resolved.addressing.is_none());
        assert_eq!(
            resolved.endpoint.as_ref(),
            "https://s3.us-west-2.amazonaws.com/"
        );
        Ok(())
    }

    #[test]
    fn s3_regular_auto_selects_virtual_for_valid_bucket() -> Result<(), Box<dyn std::error::Error>>
    {
        let resolved = s3_endpoint(cfg(Some("mybucket"), None, None, false, false))?;
        assert!(resolved.addressing.is_some());
        Ok(())
    }

    #[test]
    fn s3express_directory_bucket_uses_zonal_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let bucket = "mybucket--usw2-az1--x-s3";
        let resolved = s3_endpoint(cfg(
            Some(bucket),
            Some(AddressingStyle::Virtual),
            None,
            false,
            false,
        ))?;
        assert_eq!(
            resolved.endpoint.as_ref(),
            "https://s3express-usw2-az1.us-west-2.amazonaws.com/"
        );
        assert!(resolved.addressing.is_some());
        Ok(())
    }

    #[test]
    fn s3express_directory_bucket_auto_also_virtual() -> Result<(), Box<dyn std::error::Error>> {
        let bucket = "mybucket--use1-az4--x-s3";
        let resolved = s3_endpoint(cfg(Some(bucket), None, None, false, false))?;
        assert_eq!(
            resolved.endpoint.as_ref(),
            "https://s3express-use1-az4.us-west-2.amazonaws.com/"
        );
        assert!(resolved.addressing.is_some());
        Ok(())
    }

    #[test]
    fn s3express_directory_bucket_dualstack_uses_zonal_dualstack()
    -> Result<(), Box<dyn std::error::Error>> {
        let bucket = "mybucket--usw2-az1--x-s3";
        let resolved = s3_endpoint(cfg(Some(bucket), None, None, true, false))?;
        assert_eq!(
            resolved.endpoint.as_ref(),
            "https://s3express-usw2-az1.dualstack.us-west-2.amazonaws.com/"
        );
        Ok(())
    }

    #[test]
    fn s3express_path_style_is_rejected() {
        let bucket = "mybucket--usw2-az1--x-s3";
        let result = s3_endpoint(cfg(
            Some(bucket),
            Some(AddressingStyle::Path),
            None,
            false,
            false,
        ));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
    }

    #[test]
    fn s3express_accelerate_is_rejected() {
        let bucket = "mybucket--usw2-az1--x-s3";
        let result = s3_endpoint(cfg(Some(bucket), None, None, false, true));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
        let result = s3_endpoint(cfg(
            Some(bucket),
            Some(AddressingStyle::Virtual),
            None,
            false,
            true,
        ));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
    }

    #[test]
    fn s3express_invalid_zone_is_rejected() {
        let bucket = "mybucket--invalid--x-s3";
        let result = s3_endpoint(cfg(Some(bucket), None, None, false, false));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
    }

    #[test]
    fn s3express_dotted_bucket_is_rejected() {
        let bucket = "my.bucket--usw2-az1--x-s3";
        let result = s3_endpoint(cfg(Some(bucket), None, None, false, false));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
    }

    #[test]
    fn s3express_operation_style_overrides_client_style() -> Result<(), Box<dyn std::error::Error>>
    {
        let bucket = "mybucket--usw2-az1--x-s3";
        let resolved = s3_endpoint(cfg(
            Some(bucket),
            Some(AddressingStyle::Path),
            Some(AddressingStyle::Virtual),
            false,
            false,
        ))?;
        assert!(resolved.addressing.is_some());
        let result = s3_endpoint(cfg(
            Some(bucket),
            Some(AddressingStyle::Virtual),
            Some(AddressingStyle::Path),
            false,
            false,
        ));
        assert!(matches!(
            result,
            Err(super::super::EncodeError::InvalidEndpoint)
        ));
        Ok(())
    }

    #[test]
    fn account_prefix_not_applied_in_s3_endpoint_data_plane()
    -> Result<(), Box<dyn std::error::Error>> {
        let resolved = s3_endpoint(cfg(
            Some("mybucket"),
            Some(AddressingStyle::Virtual),
            None,
            false,
            false,
        ))?;
        assert!(resolved.endpoint.as_ref().starts_with("https://s3."));
        let bucket = "mybucket--usw2-az1--x-s3";
        let resolved = s3_endpoint(cfg(Some(bucket), None, None, false, false))?;
        assert!(resolved.endpoint.as_ref().contains("s3express-"));
        assert!(!resolved.endpoint.as_ref().contains("s3-control"));
        Ok(())
    }
}
