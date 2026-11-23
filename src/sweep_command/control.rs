use anyhow::{Context, Result};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::BinaryStore;
use inventory_sweep::{CredentialSource, SweepConfig, SweepControl, SweepSummary};

pub(crate) async fn run(
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    config: SweepConfig,
    credentials: &impl CredentialSource,
) -> Result<SweepSummary> {
    let control = SweepControl::default();
    let cancel = control.cancel.clone();
    let collection = inventory_sweep::run(catalog, store, config, credentials, control);
    tokio::pin!(collection);
    tokio::select! {
        result = &mut collection => result.map_err(anyhow::Error::from),
        signal = tokio::signal::ctrl_c() => {
            cancel.cancel();
            let result = collection.await;
            signal.context("Ctrl-C cancellation handler failed")?;
            result.map_err(anyhow::Error::from)
        }
    }
}
