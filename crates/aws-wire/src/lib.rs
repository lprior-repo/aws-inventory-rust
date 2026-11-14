//! Model-driven AWS wire engine (VISION 5.2).
//!
//! One crate sends every request the product makes: it signs with SigV4,
//! sends over one pooled client, and will serialize requests and decode
//! responses from the compiled API catalog. It owns no retry policy; the
//! executor in `inventory-engine` is the only retry owner.
#![deny(missing_docs)]

pub mod sign;
