use super::super::ProbeError;
use aws_wire::transport::{Client, Config};
use std::{fs::File, io::Read, path::Path};

pub(super) fn load(catalog: &Path, ca_bundle: &Path) -> Result<(Vec<u8>, Client), ProbeError> {
    let bytes = read_catalog(catalog)?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(ca_bundle.to_owned()),
        ..Config::default()
    })?;
    Ok((bytes, client))
}

fn read_catalog(path: &Path) -> Result<Vec<u8>, ProbeError> {
    let file = File::open(path)?;
    let length = usize::try_from(file.metadata()?.len())
        .map_err(|_| ProbeError::Invalid("probe catalog exceeds 64 MiB"))?;
    if length > 67_108_864 {
        return Err(ProbeError::Invalid("probe catalog exceeds 64 MiB"));
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length)?;
    file.take(67_108_865).read_to_end(&mut bytes)?;
    if bytes.len() > 67_108_864 {
        return Err(ProbeError::Invalid("probe catalog exceeds 64 MiB"));
    }
    Ok(bytes)
}
