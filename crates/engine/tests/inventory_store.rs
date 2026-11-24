#[path = "inventory_store/batch_seed.rs"]
mod batch_seed;
#[path = "inventory_store/behavior.rs"]
mod behavior;
#[path = "inventory_store/conformance.rs"]
mod conformance;
#[path = "inventory_store/postgres.rs"]
mod postgres;
#[path = "inventory_store/projection_fixture.rs"]
mod projection_fixture;
#[path = "inventory_store/projection_port.rs"]
mod projection_port;
#[path = "inventory_store/projection_trait.rs"]
mod projection_trait;
#[path = "inventory_store/reference.rs"]
mod reference;
#[path = "inventory_store/scalar.rs"]
mod scalar;
#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

#[tokio::test]
async fn store_conformance_binary_store() -> support::TestResult {
    conformance::run(support::conformance_fixture).await
}
