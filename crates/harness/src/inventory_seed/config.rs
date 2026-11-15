use std::{
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SeedError {
    #[error("invalid emulator seed configuration: {0}")]
    Invalid(&'static str),
    #[error("trusted CA bundle could not be read")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error(transparent)]
    RuntimeCatalog(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),
    #[error(transparent)]
    Inventory(#[from] inventory_core::inventory::InventoryError),
    #[error(transparent)]
    Encode(#[from] aws_wire::request::EncodeError),
    #[error(transparent)]
    Decode(#[from] aws_wire::response::DecodeError),
    #[error(transparent)]
    Signing(#[from] aws_wire::sign::Error),
    #[error(transparent)]
    Transport(#[from] aws_wire::transport::TransportError),
}

#[derive(Debug)]
pub struct SeedConfig {
    pub(super) endpoint: String,
    pub(super) ca_bundle: PathBuf,
    pub(super) region: inventory_core::inventory::Region,
    pub(super) permit: MutationPermit,
}

#[derive(Debug)]
pub(super) struct MutationPermit {
    _private: (),
}

impl SeedConfig {
    pub fn new(endpoint: &str, ca_bundle: &Path, region: &str) -> Result<Self, SeedError> {
        validate_endpoint(endpoint)?;
        validate_ca(ca_bundle)?;
        let region = inventory_core::inventory::Region::parse(region)
            .map_err(|_| SeedError::Invalid("region is invalid"))?;
        if matches!(region, inventory_core::inventory::Region::Global) {
            return Err(SeedError::Invalid("seed region must be regional"));
        }
        Ok(Self {
            endpoint: endpoint.to_owned(),
            ca_bundle: ca_bundle.to_owned(),
            region,
            permit: MutationPermit { _private: () },
        })
    }
}
impl SeedConfig {
    pub fn region(&self) -> &inventory_core::inventory::Region {
        &self.region
    }

    pub(super) fn mutation_permit(&self) -> &MutationPermit {
        &self.permit
    }
}

fn validate_ca(path: &Path) -> Result<(), SeedError> {
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_CA_BYTES {
        return Err(SeedError::Invalid(
            "trusted CA bundle exceeds its size or file bound",
        ));
    }
    let bytes = read_ca(file)?;
    let pem = bytes
        .windows(27)
        .any(|window| window == b"-----BEGIN CERTIFICATE-----")
        && bytes
            .windows(25)
            .any(|window| window == b"-----END CERTIFICATE-----");
    if !pem {
        return Err(SeedError::Invalid(
            "trusted CA bundle must contain PEM certificates",
        ));
    }
    Ok(())
}

const MAX_CA_BYTES: u64 = 1_048_576;
const MAX_CA_READ_BYTES: u64 = 1_048_577;

fn read_ca(file: std::fs::File) -> Result<Vec<u8>, SeedError> {
    let capacity = usize::try_from(MAX_CA_READ_BYTES)
        .map_err(|_| SeedError::Invalid("trusted CA bundle size limit is unsupported"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| SeedError::Invalid("trusted CA bundle allocation failed"))?;
    file.take(MAX_CA_READ_BYTES).read_to_end(&mut bytes)?;
    let size = u64::try_from(bytes.len())
        .map_err(|_| SeedError::Invalid("trusted CA bundle size is invalid"))?;
    if size > MAX_CA_BYTES {
        return Err(SeedError::Invalid(
            "trusted CA bundle exceeds its size or file bound",
        ));
    }
    Ok(bytes)
}

fn validate_endpoint(endpoint: &str) -> Result<(), SeedError> {
    if endpoint.len() > 255 {
        return Err(SeedError::Invalid("seed endpoint exceeds its length bound"));
    }
    let authority = endpoint
        .strip_prefix("https://")
        .ok_or(SeedError::Invalid("seed endpoint must use HTTPS"))?;
    if authority.is_empty()
        || authority
            .bytes()
            .any(|byte| matches!(byte, b'/' | b'?' | b'#' | b'@'))
    {
        return Err(SeedError::Invalid("seed endpoint authority is invalid"));
    }
    let (host, port) = authority.rsplit_once(':').ok_or(SeedError::Invalid(
        "seed endpoint requires an explicit port",
    ))?;
    validate_loopback(host)?;
    validate_port(port)
}

fn validate_port(port: &str) -> Result<(), SeedError> {
    let parsed = port
        .parse::<u16>()
        .map_err(|_| SeedError::Invalid("seed endpoint port is invalid"))?;
    if parsed == 0 || port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SeedError::Invalid("seed endpoint port is invalid"));
    }
    Ok(())
}

fn validate_loopback(host: &str) -> Result<(), SeedError> {
    let ipv6 = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'));
    let valid = ipv6
        .and_then(|value| value.parse::<Ipv6Addr>().ok())
        .is_some_and(|address| address.is_loopback())
        || host
            .parse::<Ipv4Addr>()
            .is_ok_and(|address| address.is_loopback());
    if valid {
        Ok(())
    } else {
        Err(SeedError::Invalid("seed endpoint must be loopback"))
    }
}
