#![cfg(feature = "aws")]

static E2E_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
#[path = "sweep_e2e/barrier_proxy.rs"]
mod barrier_proxy;
#[path = "sweep_e2e/denial_peer.rs"]
mod denial_peer;
#[path = "sweep_e2e/guard.rs"]
mod guard;
#[path = "sweep_e2e/process.rs"]
mod process;
#[path = "sweep_e2e/scenario.rs"]
mod scenario;
#[path = "sweep_e2e/state.rs"]
mod state;

#[tokio::test]
#[ignore = "requires explicit loopback emulator, trusted CA, and loopback FalkorDB"]
async fn production_sweep_persists_and_projects_real_seeded_inventory() -> anyhow::Result<()> {
    let _guard = E2E_LOCK.lock().await;
    scenario::run_live().await
}

#[tokio::test]
#[ignore = "requires explicit loopback emulator, trusted CA, and profile safeguard environment"]
async fn controlled_tls_403_preserves_existing_inventory() -> anyhow::Result<()> {
    let _guard = E2E_LOCK.lock().await;
    scenario::run_denied().await
}
