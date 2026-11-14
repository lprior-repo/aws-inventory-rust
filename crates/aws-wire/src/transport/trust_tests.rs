use super::assemble;
use rustls::{
    RootCertStore,
    client::{WebPkiServerVerifier, danger::ServerCertVerifier},
};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn extra_bundle_preserves_and_validates_a_distinct_native_root() -> TestResult {
    let native = rcgen::generate_simple_self_signed(vec!["native.test".to_owned()])?;
    let extra = rcgen::generate_simple_self_signed(vec!["extra.test".to_owned()])?;
    let mut loaded = rustls_native_certs::CertificateResult::default();
    loaded.certs.push(native.cert.der().clone());

    let roots = Arc::new(assemble(loaded, Some(extra.cert.pem().as_bytes()))?);
    assert_eq!(roots.len(), 2);

    verify(Arc::clone(&roots), native.cert.der(), "native.test")?;
    verify(roots, extra.cert.der(), "extra.test")
}

fn verify(roots: Arc<RootCertStore>, cert: &CertificateDer<'_>, name: &str) -> TestResult {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let verifier = WebPkiServerVerifier::builder_with_provider(roots, provider).build()?;
    verifier.verify_server_cert(
        cert,
        &[],
        &ServerName::try_from(name)?,
        &[],
        UnixTime::now(),
    )?;
    Ok(())
}
