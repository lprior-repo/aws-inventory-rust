use super::super::DecodeError;
use super::{
    decoder::{Frame, count_node},
    scalar,
    schema::invalid_cbor,
};
use minicbor::{Decoder, data::Type};

pub(super) fn parse_unknown(
    decoder: &mut Decoder<'_>,
    frames: &mut Vec<Frame>,
    nodes: &mut usize,
    max_nodes: usize,
    max_depth: usize,
) -> Result<(), DecodeError> {
    loop {
        match decoder.datatype().map_err(|_| invalid_cbor())? {
            Type::Tag => {
                decoder.tag().map_err(|_| invalid_cbor())?;
                count_node(nodes, max_nodes)?;
            }
            _ => return open_unknown(decoder, frames, max_depth),
        }
    }
}

fn open_unknown(
    decoder: &mut Decoder<'_>,
    frames: &mut Vec<Frame>,
    depth: usize,
) -> Result<(), DecodeError> {
    let (map, remaining) = match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::Map | Type::MapIndef => (true, decoder.map().map_err(|_| invalid_cbor())?),
        Type::Array | Type::ArrayIndef => (false, decoder.array().map_err(|_| invalid_cbor())?),
        _ => return scalar::skip_scalar(decoder),
    };
    push_unknown(
        frames,
        Frame {
            map,
            remaining,
            target: None,
            modeled: false,
            seen: Vec::new(),
        },
        depth,
    )
}

fn push_unknown(
    frames: &mut Vec<Frame>,
    frame: Frame,
    max_depth: usize,
) -> Result<(), DecodeError> {
    if frames.len() >= max_depth {
        return Err(DecodeError::Limit("response depth"));
    }
    frames.try_reserve(1)?;
    frames.push(frame);
    Ok(())
}
