use super::SeedError;
use std::{collections::BTreeMap, path::PathBuf};

pub struct SeedRunArgs {
    pub seeds: PathBuf,
    pub catalog: PathBuf,
    pub endpoint: String,
    pub ca_bundle: PathBuf,
    pub region: String,
    pub services: Vec<String>,
    pub oracle: PathBuf,
    pub matrix: PathBuf,
}

impl SeedRunArgs {
    pub fn parse(arguments: &[String]) -> Result<Self, SeedError> {
        if arguments.len() > 128 || arguments.len() % 2 != 0 {
            return Err(SeedError::Invalid("seed run requires at most 64 flag/value pairs"));
        }
        let (options, services) = options(arguments)?;
        let required = |name| options.get(name).cloned()
            .ok_or_else(|| SeedError::Message(format!("seed run requires {name}")));
        let args = Self {
            seeds: required("--seeds")?.into(), catalog: required("--catalog")?.into(),
            endpoint: required("--endpoint")?, ca_bundle: required("--ca-bundle")?.into(),
            region: required("--region")?, oracle: required("--oracle")?.into(),
            matrix: required("--matrix")?.into(), services,
        };
        if args.oracle == args.matrix || args.oracle == args.catalog || args.matrix == args.catalog {
            return Err(SeedError::Invalid("seed report paths must differ from each other and the catalog"));
        }
        Ok(args)
    }
}

type Options = (BTreeMap<String, String>, Vec<String>);
fn options(arguments: &[String]) -> Result<Options, SeedError> {
    arguments.chunks_exact(2).try_fold((BTreeMap::new(), Vec::new()), |(mut options, mut services), pair| {
        let [flag, value] = pair else { return Err(SeedError::Invalid("seed option has no value")); };
        if value.is_empty() || value.len() > 4096 {
            return Err(SeedError::Invalid("seed option value is empty or oversized"));
        }
        if flag == "--service" {
            services.push(value.clone());
        } else {
            if !["--seeds", "--catalog", "--endpoint", "--ca-bundle", "--region", "--oracle", "--matrix"].contains(&flag.as_str())
                || options.insert(flag.clone(), value.clone()).is_some() {
                return Err(SeedError::Message(format!("unknown or repeated seed flag: {flag}")));
            }
        }
        Ok((options, services))
    })
}
