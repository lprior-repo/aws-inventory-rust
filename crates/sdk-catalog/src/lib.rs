//! Generated SDK catalog. Availability is not implemented collector coverage.
//! Service crates compile only when their explicit feature is enabled.
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub const SERVICE_COUNT: usize = 437;
pub const SERVICE_MANIFEST_JSON: &str = include_str!("../sdk-services.json");

#[cfg(feature = "catalog-a")]
mod services_a;
#[cfg(feature = "catalog-a")]
pub use services_a::*;

#[cfg(feature = "catalog-b")]
mod services_b;
#[cfg(feature = "catalog-b")]
pub use services_b::*;

#[cfg(feature = "catalog-c")]
mod services_c;
#[cfg(feature = "catalog-c")]
pub use services_c::*;

#[cfg(feature = "catalog-d")]
mod services_d;
#[cfg(feature = "catalog-d")]
pub use services_d::*;

#[cfg(feature = "catalog-e")]
mod services_e;
#[cfg(feature = "catalog-e")]
pub use services_e::*;

#[cfg(feature = "catalog-f")]
mod services_f;
#[cfg(feature = "catalog-f")]
pub use services_f::*;

#[cfg(feature = "catalog-g")]
mod services_g;
#[cfg(feature = "catalog-g")]
pub use services_g::*;

#[cfg(feature = "catalog-h")]
mod services_h;
#[cfg(feature = "catalog-h")]
pub use services_h::*;

#[cfg(feature = "catalog-i")]
mod services_i;
#[cfg(feature = "catalog-i")]
pub use services_i::*;

#[cfg(feature = "catalog-k")]
mod services_k;
#[cfg(feature = "catalog-k")]
pub use services_k::*;

#[cfg(feature = "catalog-l")]
mod services_l;
#[cfg(feature = "catalog-l")]
pub use services_l::*;

#[cfg(feature = "catalog-m")]
mod services_m;
#[cfg(feature = "catalog-m")]
pub use services_m::*;

#[cfg(feature = "catalog-n")]
mod services_n;
#[cfg(feature = "catalog-n")]
pub use services_n::*;

#[cfg(feature = "catalog-o")]
mod services_o;
#[cfg(feature = "catalog-o")]
pub use services_o::*;

#[cfg(feature = "catalog-p")]
mod services_p;
#[cfg(feature = "catalog-p")]
pub use services_p::*;

#[cfg(feature = "catalog-q")]
mod services_q;
#[cfg(feature = "catalog-q")]
pub use services_q::*;

#[cfg(feature = "catalog-r")]
mod services_r;
#[cfg(feature = "catalog-r")]
pub use services_r::*;

#[cfg(feature = "catalog-s")]
mod services_s;
#[cfg(feature = "catalog-s")]
pub use services_s::*;

#[cfg(feature = "catalog-t")]
mod services_t;
#[cfg(feature = "catalog-t")]
pub use services_t::*;

#[cfg(feature = "catalog-u")]
mod services_u;
#[cfg(feature = "catalog-u")]
pub use services_u::*;

#[cfg(feature = "catalog-v")]
mod services_v;
#[cfg(feature = "catalog-v")]
pub use services_v::*;

#[cfg(feature = "catalog-w")]
mod services_w;
#[cfg(feature = "catalog-w")]
pub use services_w::*;

#[cfg(feature = "catalog-x")]
mod services_x;
#[cfg(feature = "catalog-x")]
pub use services_x::*;
