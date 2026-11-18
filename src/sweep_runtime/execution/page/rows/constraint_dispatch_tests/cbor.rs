use super::TestResult;
use serde_json::Value as Json;

pub(super) fn from_json(value: &Json) -> TestResult<Vec<u8>> {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    write(&mut encoder, value)?;
    Ok(encoder.into_writer())
}

pub(super) fn to_json(bytes: &[u8]) -> TestResult<Json> {
    let mut decoder = minicbor::Decoder::new(bytes);
    let value = read(&mut decoder)?;
    if decoder.position() != bytes.len() {
        return Err("trailing CBOR bytes".into());
    }
    Ok(value)
}

fn write(encoder: &mut minicbor::Encoder<Vec<u8>>, value: &Json) -> TestResult {
    match value {
        Json::Null => {
            encoder.null()?;
        }
        Json::Bool(flag) => {
            encoder.bool(*flag)?;
        }
        Json::Number(number) => match (number.as_i64(), number.as_u64()) {
            (Some(signed), _) => {
                encoder.i64(signed)?;
            }
            (None, Some(unsigned)) => {
                encoder.u64(unsigned)?;
            }
            (None, None) => return Err("fixture number is not an integer".into()),
        },
        Json::String(text) => {
            encoder.str(text)?;
        }
        Json::Array(items) => {
            encoder.array(u64::try_from(items.len())?)?;
            for item in items {
                write(encoder, item)?;
            }
        }
        Json::Object(members) => {
            encoder.map(u64::try_from(members.len())?)?;
            for (name, member) in members {
                encoder.str(name)?;
                write(encoder, member)?;
            }
        }
    }
    Ok(())
}

fn read(decoder: &mut minicbor::Decoder<'_>) -> TestResult<Json> {
    use minicbor::data::Type;
    Ok(match decoder.datatype()? {
        Type::Null => {
            decoder.null()?;
            Json::Null
        }
        Type::Bool => Json::Bool(decoder.bool()?),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => Json::from(decoder.u64()?),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => Json::from(decoder.i64()?),
        Type::String => Json::from(decoder.str()?),
        Type::Array | Type::ArrayIndef => {
            let mut remaining = decoder.array()?;
            let mut items = Vec::new();
            while more(decoder, &mut remaining)? {
                items.push(read(decoder)?);
            }
            Json::Array(items)
        }
        Type::Map | Type::MapIndef => {
            let mut remaining = decoder.map()?;
            let mut members = serde_json::Map::new();
            while more(decoder, &mut remaining)? {
                let name = decoder.str()?.to_owned();
                members.insert(name, read(decoder)?);
            }
            Json::Object(members)
        }
        other => return Err(format!("unsupported CBOR type in fixture: {other:?}").into()),
    })
}

fn more(decoder: &mut minicbor::Decoder<'_>, remaining: &mut Option<u64>) -> TestResult<bool> {
    match remaining {
        Some(0) => Ok(false),
        Some(count) => {
            *count -= 1;
            Ok(true)
        }
        None if decoder.datatype()? == minicbor::data::Type::Break => {
            decoder.skip()?;
            Ok(false)
        }
        None => Ok(true),
    }
}
