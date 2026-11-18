use minicbor::{Decoder, Encoder, data::Type};
use serde_json::{Map, Value as Json};

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(crate) fn json_to_cbor(value: &Json) -> TestResult<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new());
    encode(&mut encoder, value)?;
    Ok(encoder.into_writer())
}

fn encode(encoder: &mut Encoder<Vec<u8>>, value: &Json) -> TestResult<()> {
    match value {
        Json::Null => {
            encoder.null()?;
        }
        Json::Bool(flag) => {
            encoder.bool(*flag)?;
        }
        Json::Number(number) => match (number.as_i64(), number.as_u64(), number.as_f64()) {
            (Some(signed), _, _) => {
                encoder.i64(signed)?;
            }
            (None, Some(unsigned), _) => {
                encoder.u64(unsigned)?;
            }
            (None, None, Some(float)) => {
                encoder.f64(float)?;
            }
            _ => return Err("unsupported JSON number".into()),
        },
        Json::String(text) => {
            encoder.str(text)?;
        }
        Json::Array(items) => {
            encoder.array(u64::try_from(items.len())?)?;
            for item in items {
                encode(encoder, item)?;
            }
        }
        Json::Object(map) => {
            encoder.map(u64::try_from(map.len())?)?;
            for (key, item) in map {
                encoder.str(key)?;
                encode(encoder, item)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn cbor_to_json(bytes: &[u8]) -> TestResult<Json> {
    let mut decoder = Decoder::new(bytes);
    let value = decode(&mut decoder)?;
    if decoder.position() != bytes.len() {
        return Err("trailing CBOR bytes".into());
    }
    Ok(value)
}

fn decode(decoder: &mut Decoder<'_>) -> TestResult<Json> {
    Ok(match decoder.datatype()? {
        Type::Null | Type::Undefined => {
            decoder.skip()?;
            Json::Null
        }
        Type::Bool => Json::Bool(decoder.bool()?),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => Json::from(decoder.u64()?),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => Json::from(decoder.i64()?),
        Type::F16 | Type::F32 | Type::F64 => Json::from(decoder.f64()?),
        Type::String => Json::String(decoder.str()?.to_owned()),
        Type::Array | Type::ArrayIndef => {
            let mut items = Vec::new();
            let length = decoder.array()?;
            let mut seen = 0u64;
            while more(decoder, length, seen)? {
                items.push(decode(decoder)?);
                seen += 1;
            }
            Json::Array(items)
        }
        Type::Map | Type::MapIndef => {
            let mut map = Map::new();
            let length = decoder.map()?;
            let mut seen = 0u64;
            while more(decoder, length, seen)? {
                let key = decoder.str()?.to_owned();
                map.insert(key, decode(decoder)?);
                seen += 1;
            }
            Json::Object(map)
        }
        other => return Err(format!("unsupported CBOR type {other}").into()),
    })
}

fn more(decoder: &mut Decoder<'_>, length: Option<u64>, seen: u64) -> TestResult<bool> {
    match length {
        Some(total) => Ok(seen < total),
        None if decoder.datatype()? == Type::Break => {
            decoder.skip()?;
            Ok(false)
        }
        None => Ok(true),
    }
}
