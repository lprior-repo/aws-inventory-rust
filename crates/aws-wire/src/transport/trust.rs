use super::{Config, TransportError};
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, pem::PemObject};
use std::{fs::File, io::Read, path::Path, sync::Arc};

pub(super) fn load(config: &Config) -> Result<ClientConfig, TransportError> {
    let pem = config
        .extra_ca_bundle
        .as_deref()
        .map(read_bundle)
        .transpose()?;
    let roots = assemble(rustls_native_certs::load_native_certs(), pem.as_deref())?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Ok(ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| TransportError::Tls(Box::new(e)))?
        .with_root_certificates(roots)
        .with_no_client_auth())
}

fn assemble(
    native: rustls_native_certs::CertificateResult,
    extra: Option<&[u8]>,
) -> Result<RootCertStore, TransportError> {
    if let Some(error) = native.errors.into_iter().next() {
        return Err(TransportError::Tls(Box::new(error)));
    }
    let mut roots = RootCertStore::empty();
    native
        .certs
        .into_iter()
        .try_for_each(|cert| roots.add(cert))
        .map_err(|e| TransportError::Tls(Box::new(e)))?;
    if let Some(pem) = extra {
        append(&mut roots, pem)?;
    }
    Ok(roots)
}

fn append(roots: &mut RootCertStore, pem: &[u8]) -> Result<(), TransportError> {
    let before = roots.len();
    CertificateDer::pem_slice_iter(pem).try_for_each(|cert| {
        roots
            .add(cert.map_err(|e| TransportError::Tls(Box::new(e)))?)
            .map_err(|e| TransportError::Tls(Box::new(e)))
    })?;
    if roots.len() == before {
        return Err(TransportError::InvalidRequest(
            "CA bundle contains no certificates",
        ));
    }
    Ok(())
}

fn read_bundle(path: &Path) -> Result<Vec<u8>, TransportError> {
    let file = File::open(path).map_err(|e| TransportError::Io(Box::new(e)))?;
    let mut pem = Vec::new();
    pem.try_reserve_exact(1_048_577)
        .map_err(|e| TransportError::Io(Box::new(e)))?;
    file.take(1_048_577)
        .read_to_end(&mut pem)
        .map_err(|e| TransportError::Io(Box::new(e)))?;
    if pem.len() > 1_048_576 {
        return Err(TransportError::InvalidRequest("CA bundle exceeds 1 MiB"));
    }
    Ok(pem)
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;
