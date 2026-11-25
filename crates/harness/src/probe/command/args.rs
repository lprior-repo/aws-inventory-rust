use super::super::ProbeError;
use inventory_sweep::ChildProbeSamples;
use std::{net::IpAddr, path::PathBuf};

pub(super) struct Args {
    pub catalog: PathBuf,
    pub endpoint: String,
    pub ca_bundle: PathBuf,
    pub region: String,
    pub report: PathBuf,
    pub services: Vec<String>,
    pub child_samples: ChildProbeSamples,
    pub pairs: Vec<String>,
    pub compared_at: Option<String>,
}

impl Args {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, ProbeError> {
        if arguments.len() < 10 || arguments.len() > 524 || !arguments.len().is_multiple_of(2) {
            return Err(ProbeError::Invalid(
                "probe requires five option-value pairs plus optional service, sample, or comparison pairs",
            ));
        }
        let options = arguments.as_chunks::<2>().0.iter().try_fold(
            Options::default(),
            |mut options, [name, value]| {
                options.set(name, value)?;
                Ok::<_, ProbeError>(options)
            },
        )?;
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
    child_samples: Option<&'a str>,
    services: Vec<&'a str>,
    pairs: Vec<&'a str>,
    compared_at: Option<&'a str>,
}

impl<'a> Options<'a> {
    fn set(&mut self, name: &str, value: &'a str) -> Result<(), ProbeError> {
        if value.is_empty() || value.len() > 4096 {
            return Err(ProbeError::Invalid(
                "probe option value is empty or too long",
            ));
        }
        if name == "--compare-pair" {
            self.pairs.try_reserve(1)?;
            self.pairs.push(value);
            return Ok(());
        }
        if name == "--service" {
            if self.services.len() == 256 {
                return Err(ProbeError::Invalid(
                    "probe accepts at most 256 service options",
                ));
            }
            self.services.try_reserve(1)?;
            self.services.push(value);
            return Ok(());
        }
        let slot = match name {
            "--catalog" => &mut self.catalog,
            "--endpoint" => &mut self.endpoint,
            "--ca-bundle" => &mut self.ca_bundle,
            "--region" => &mut self.region,
            "--report" => &mut self.report,
            "--child-samples" => &mut self.child_samples,
            "--compared-at" => &mut self.compared_at,
            _ => return Err(ProbeError::Invalid("unknown probe option")),
        };
        if slot.replace(value).is_some() {
            return Err(ProbeError::Invalid("duplicate probe option"));
        }
        Ok(())
    }

    fn finish(self) -> Result<Args, ProbeError> {
        if !self.services.is_empty() && !self.pairs.is_empty() {
            return Err(ProbeError::Invalid(
                "--service cannot be combined with --compare-pair",
            ));
        }
        if self.child_samples.is_some() && !self.pairs.is_empty() {
            return Err(ProbeError::Invalid(
                "--child-samples cannot be combined with --compare-pair",
            ));
        }
        let args = Args {
            catalog: required(self.catalog)?.into(),
            endpoint: required(self.endpoint)?.into(),
            ca_bundle: required(self.ca_bundle)?.into(),
            region: required(self.region)?.into(),
            report: required(self.report)?.into(),
            services: self.services.into_iter().map(str::to_owned).collect(),
            child_samples: self.child_samples.map_or_else(
                || Ok(ChildProbeSamples::default()),
                |value| {
                    value
                        .parse::<usize>()
                        .ok()
                        .and_then(ChildProbeSamples::new)
                        .ok_or(ProbeError::Invalid(
                            "--child-samples requires a count from 1 through 64",
                        ))
                },
            )?,
            pairs: self.pairs.into_iter().map(str::to_owned).collect(),
            compared_at: self.compared_at.map(str::to_owned),
        };
        endpoint(&args.endpoint)?;
        if args.pairs.is_empty() != args.compared_at.is_none() {
            return Err(ProbeError::Invalid(
                "compare-pair and compared-at must be supplied together",
            ));
        }
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
