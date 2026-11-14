use minicbor::{
    Decoder,
    data::{Int, Type},
};

const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_ITEMS: usize = 100_000;
const MAX_DEPTH: usize = 64;

#[derive(Debug)]
enum Cbor {
    Int(Int),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Cbor>),
    Map(Vec<(Cbor, Cbor)>),
    Tag(u64, Box<Cbor>),
    Bool(bool),
    Null,
    Undefined,
    Simple(u8),
}

impl PartialEq for Cbor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => {
                left == right || left.is_nan() && right.is_nan()
            }
            (Self::Bytes(left), Self::Bytes(right)) => left == right,
            (Self::Text(left), Self::Text(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => left == right,
            (Self::Map(left), Self::Map(right)) => {
                left.len() == right.len()
                    && left.iter().all(|(key, value)| {
                        right.iter().any(|(other_key, other_value)| {
                            key == other_key && value == other_value
                        })
                    })
            }
            (Self::Tag(left_tag, left), Self::Tag(right_tag, right)) => {
                left_tag == right_tag
                    && if *left_tag == 1 {
                        timestamp_equal(left, right)
                    } else {
                        left == right
                    }
            }
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Simple(left), Self::Simple(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Cbor {}

fn timestamp_equal(left: &Cbor, right: &Cbor) -> bool {
    match (left, right) {
        (Cbor::Int(integer), Cbor::Float(float)) | (Cbor::Float(float), Cbor::Int(integer)) => {
            float.is_finite()
                && float.fract() == 0.0
                && format!("{float:.0}").parse::<i128>().ok() == Some(i128::from(*integer))
        }
        _ => left == right,
    }
}

fn rpc_data(value: &str) -> Cbor {
    Cbor::Map(vec![(Cbor::Text("data".into()), Cbor::Text(value.into()))])
}

pub(super) fn assert_rpc_data(body: &[u8], value: &str, label: &str) {
    assert_eq!(
        decode(body, label),
        rpc_data(value),
        "{label} RPCv2 CBOR data parity"
    );
}

pub(super) fn assert_semantically_equal(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(
        decode(actual, label),
        decode(expected, label),
        "{label} CBOR semantic equality"
    );
}

fn decode(input: &[u8], label: &str) -> Cbor {
    assert!(
        input.len() <= MAX_BYTES,
        "{label}: CBOR input exceeds processing bound"
    );
    let mut decoder = Decoder::new(input);
    let mut budget = MAX_ITEMS;
    let value = parse_value(&mut decoder, 0, &mut budget)
        .unwrap_or_else(|error| panic!("{label}: invalid CBOR: {error}"));
    assert_eq!(
        decoder.position(),
        input.len(),
        "{label}: trailing CBOR data"
    );
    value
}

fn parse_value(
    decoder: &mut Decoder<'_>,
    depth: usize,
    budget: &mut usize,
) -> Result<Cbor, String> {
    if depth > MAX_DEPTH || *budget == 0 {
        return Err("CBOR nesting/item bound exceeded".into());
    }
    *budget -= 1;
    let kind = decoder.datatype().map_err(|error| error.to_string())?;
    parse_kind(decoder, depth, budget, kind)
}

fn parse_kind(
    decoder: &mut Decoder<'_>,
    depth: usize,
    budget: &mut usize,
    kind: Type,
) -> Result<Cbor, String> {
    match kind {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::Int => parse_integer(decoder),
        Type::F16 => Ok(Cbor::Float(f64::from(
            decoder.f16().map_err(|error| error.to_string())?,
        ))),
        Type::F32 => Ok(Cbor::Float(f64::from(
            decoder.f32().map_err(|error| error.to_string())?,
        ))),
        Type::F64 => Ok(Cbor::Float(
            decoder.f64().map_err(|error| error.to_string())?,
        )),
        Type::Bool => Ok(Cbor::Bool(
            decoder.bool().map_err(|error| error.to_string())?,
        )),
        Type::Null => parse_null(decoder),
        Type::Undefined => parse_undefined(decoder),
        Type::Simple => Ok(Cbor::Simple(
            decoder.simple().map_err(|error| error.to_string())?,
        )),
        Type::Bytes => Ok(Cbor::Bytes(
            decoder.bytes().map_err(|error| error.to_string())?.to_vec(),
        )),
        Type::BytesIndef => parse_bytes(decoder),
        Type::String => Ok(Cbor::Text(
            decoder.str().map_err(|error| error.to_string())?.to_owned(),
        )),
        Type::StringIndef => parse_text(decoder),
        Type::Array | Type::ArrayIndef => parse_array(decoder, depth, budget),
        Type::Map | Type::MapIndef => parse_map(decoder, depth, budget),
        Type::Tag => parse_tag(decoder, depth, budget),
        _ => Err(format!("unsupported CBOR type {kind:?}")),
    }
}

fn parse_integer(decoder: &mut Decoder<'_>) -> Result<Cbor, String> {
    Ok(Cbor::Int(decoder.int().map_err(|error| error.to_string())?))
}

fn parse_null(decoder: &mut Decoder<'_>) -> Result<Cbor, String> {
    decoder.null().map_err(|error| error.to_string())?;
    Ok(Cbor::Null)
}

fn parse_undefined(decoder: &mut Decoder<'_>) -> Result<Cbor, String> {
    decoder.undefined().map_err(|error| error.to_string())?;
    Ok(Cbor::Undefined)
}

fn parse_tag(decoder: &mut Decoder<'_>, depth: usize, budget: &mut usize) -> Result<Cbor, String> {
    let tag = decoder.tag().map_err(|error| error.to_string())?.as_u64();
    Ok(Cbor::Tag(
        tag,
        Box::new(parse_value(decoder, depth + 1, budget)?),
    ))
}

fn parse_array(
    decoder: &mut Decoder<'_>,
    depth: usize,
    budget: &mut usize,
) -> Result<Cbor, String> {
    let length = decoder.array().map_err(|error| error.to_string())?;
    let mut values = Vec::new();
    while has_item(decoder, length, values.len())? {
        values.push(parse_value(decoder, depth + 1, budget)?);
    }
    Ok(Cbor::Array(values))
}

fn parse_map(decoder: &mut Decoder<'_>, depth: usize, budget: &mut usize) -> Result<Cbor, String> {
    let length = decoder.map().map_err(|error| error.to_string())?;
    let mut entries: Vec<(Cbor, Cbor)> = Vec::new();
    while has_item(decoder, length, entries.len())? {
        let key = parse_value(decoder, depth + 1, budget)?;
        if entries.iter().any(|(existing, _)| *existing == key) {
            return Err("duplicate CBOR map key".into());
        }
        if decoder.datatype().map_err(|error| error.to_string())? == Type::Break {
            return Err("CBOR map key has no value".into());
        }
        let value = parse_value(decoder, depth + 1, budget)?;
        entries.push((key, value));
    }
    Ok(Cbor::Map(entries))
}

fn has_item(decoder: &mut Decoder<'_>, length: Option<u64>, count: usize) -> Result<bool, String> {
    match length {
        Some(length) => Ok(u64::try_from(count).map_err(|error| error.to_string())? < length),
        None if decoder.datatype().map_err(|error| error.to_string())? == Type::Break => {
            decoder.skip().map_err(|error| error.to_string())?;
            Ok(false)
        }
        None => Ok(true),
    }
}

fn parse_bytes(decoder: &mut Decoder<'_>) -> Result<Cbor, String> {
    let mut bytes = Vec::new();
    loop {
        if decoder.datatype().map_err(|error| error.to_string())? == Type::Break {
            decoder.skip().map_err(|error| error.to_string())?;
            break;
        }
        bytes.extend_from_slice(decoder.bytes().map_err(|error| error.to_string())?);
        if bytes.len() > MAX_BYTES {
            return Err("CBOR bytes exceed processing bound".into());
        }
    }
    Ok(Cbor::Bytes(bytes))
}

fn parse_text(decoder: &mut Decoder<'_>) -> Result<Cbor, String> {
    let mut text = String::new();
    loop {
        if decoder.datatype().map_err(|error| error.to_string())? == Type::Break {
            decoder.skip().map_err(|error| error.to_string())?;
            break;
        }
        text.push_str(decoder.str().map_err(|error| error.to_string())?);
        if text.len() > MAX_BYTES {
            return Err("CBOR text exceeds processing bound".into());
        }
    }
    Ok(Cbor::Text(text))
}

#[cfg(test)]
#[path = "cbor_semantics_tests.rs"]
mod tests;
