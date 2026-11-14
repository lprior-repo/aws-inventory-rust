use super::{DecodeError, ResponseContext};
use inventory_core::record::Record;

mod decode;
mod http;
mod index;
mod model;
mod scalar;

pub fn decode(context: ResponseContext<'_>) -> Result<Record, DecodeError> {
    decode::decode(context)
}
