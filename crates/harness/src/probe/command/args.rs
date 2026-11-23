use super::super::ProbeError;
use std::{net::IpAddr, path::PathBuf};

pub(super) struct Args {
    pub catalog: PathBuf,
    pub endpoint: String,
    pub ca_bundle: PathBuf,
    pub region: String,
    pub report: PathBuf,
}

impl Args {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, ProbeError> {
        if arguments.len() != 10 {
            return Err(ProbeError::Invalid(
                "probe requires exactly five option-value pairs",
            ));
        }
        let mut options = Options::default();
        for [name, value] in arguments.as_chunks::<2>().0 {
            options.set(name, value)?;
        }
        options.finish()
    }
}

#[derive(Default)]
struct Options<'a> {
    catalog: Option<&'a str>,
    endpoint: Option<&'a str>,
    ca_bundle: Option<&'a str>,
    region: Option<&'a str>,
    report: Option<&'a str>,
}

impl<'a> Options<'a> {
    fn set(&mut self, name: &str, value: &'a str) -> Result<(), ProbeError> {
        if value.is_empty() || value.len() > 4096 {
            return Err(ProbeError::Invalid(
                "probe option value is empty or too long",
            ));
        }
        let slot = match name {
            "--catalog" => &mut self.catalog,
            "--endpoint" => &mut self.endpoint,
            "--ca-bundle" => &mut self.ca_bundle,
            "--region" => &mut self.region,
            "--report" => &mut self.report,
            _ => return Err(ProbeError::Invalid("unknown probe option")),
        };
        if slot.replace(value).is_some() {
            return Err(ProbeError::Invalid("duplicate probe option"));
        }
        Ok(())
    }

    fn finish(self) -> Result<Args, ProbeError> {
        let args = Args {
            catalog: required(self.catalog)?.into(),
            endpoint: required(self.endpoint)?.into(),
            ca_bundle: required(self.ca_bundle)?.into(),
            region: required(self.region)?.into(),
            report: required(self.report)?.into(),
        };
        endpoint(&args.endpoint)?;
        Ok(args)
    }
}

fn required(value: Option<&str>) -> Result<&str, ProbeError> {
    value.ok_or(ProbeError::Invalid("missing required probe option"))
}

fn endpoint(value: &str) -> Result<(), ProbeError> {
    let uri: http::Uri = value
        .parse()
        .map_err(|_| ProbeError::Invalid("invalid probe endpoint"))?;
    if value.len() > 255
        || uri.scheme_str() != Some("https")
        || !loopback(&uri)
        || !uri.port_u16().is_some_and(|port| port != 0)
        || !origin(value)
    {
        return Err(ProbeError::Invalid(
            "probe endpoint requires a loopback HTTPS origin with a port",
        ));
    }
    Ok(())
}

fn loopback(uri: &http::Uri) -> bool {
    uri.host().is_some_and(|host| {
        let host = host
            .strip_prefix('[')
            .and_then(|host| host.strip_suffix(']'))
            .unwrap_or(host);
        host.parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
    })
}

fn origin(value: &str) -> bool {
    value.strip_prefix("https://").is_some_and(|authority| {
        !authority
            .bytes()
            .any(|byte| matches!(byte, b'/' | b'?' | b'#' | b'@'))
    })
}

#[cfg(test)]
mod tests;
