use super::*;
use scalar::ScalarContext;

pub(super) fn open_child_map(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    frames: &mut Vec<Frame>,
    value: ScalarContext<'_>,
    depth: usize,
) -> Result<(), DecodeError> {
    let modeled = matches!(value.kind, Some("structure" | "union" | "map" | "document"));
    if value.target.is_some() && !modeled {
        return Err(DecodeError::Invalid(
            "CBOR map does not match modeled shape",
        ));
    }
    let count = decoder.map().map_err(|_| invalid_cbor())?;
    if modeled {
        builder.begin_map(value.key)?;
    }
    push_frame(frames, map_frame(value, count, modeled), depth)
}

fn map_frame(value: ScalarContext<'_>, count: Option<u64>, modeled: bool) -> Frame {
    Frame {
        map: true,
        remaining: count,
        target: child_target(&value),
        modeled,
        seen: Vec::new(),
    }
}

pub(super) fn open_child_list(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    frames: &mut Vec<Frame>,
    value: ScalarContext<'_>,
    depth: usize,
) -> Result<(), DecodeError> {
    if value.target.is_some() && !matches!(value.kind, Some("list" | "document")) {
        return Err(DecodeError::Invalid(
            "CBOR array does not match modeled shape",
        ));
    }
    let count = decoder.array().map_err(|_| invalid_cbor())?;
    if value.target.is_some() {
        builder.begin_list(value.key)?;
    }
    push_frame(frames, list_frame(value, count), depth)
}

fn list_frame(value: ScalarContext<'_>, count: Option<u64>) -> Frame {
    Frame {
        map: false,
        remaining: count,
        target: child_target(&value),
        modeled: value.target.is_some(),
        seen: Vec::new(),
    }
}

fn child_target(value: &ScalarContext<'_>) -> Option<Target> {
    if value.kind == Some("document") {
        Some(Target::Prelude("document"))
    } else {
        value.target
    }
}

fn push_frame(frames: &mut Vec<Frame>, frame: Frame, depth: usize) -> Result<(), DecodeError> {
    if frames.len() >= depth {
        return Err(DecodeError::Limit("response depth"));
    }
    frames.try_reserve(1)?;
    frames.push(frame);
    Ok(())
}
