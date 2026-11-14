mod decoder;
mod scalar;
mod schema;
mod unknown;

use super::{DecodeError, ResponseContext};

pub fn decode(context: ResponseContext<'_>) -> Result<inventory_core::record::Record, DecodeError> {
    if !context.status.is_success() {
        return Err(DecodeError::Invalid(
            "non-success response belongs to error decoder",
        ));
    }
    if context.protocol != crate::request::Protocol::RpcV2Cbor {
        return Err(DecodeError::Unsupported("CBOR response protocol"));
    }
    decoder::decode(context)
}
