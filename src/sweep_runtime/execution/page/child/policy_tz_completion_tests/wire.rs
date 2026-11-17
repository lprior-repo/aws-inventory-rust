use super::{Json, TestResult};
use inventory_core::catalog::{Service, Target};

pub(super) fn json_to_cbor(value: &Json, service: Service<'_>, output_target: Target) -> TestResult<Vec<u8>> {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encode_json_with_schema(&mut encoder, value, service, Some(output_target))?;
    Ok(encoder.into_writer())
}

fn resolve_target_kind(service: Service<'_>, target: Target) -> TestResult<&'static str> {
    match target {
        Target::Shape(index) => {
            let shape = service.shape(index)?.ok_or("modeled CBOR shape absent")?;
            Ok(shape.kind()?)
        }
        Target::Prelude(kind) => Ok(kind),
        _ => Err("unsupported modeled CBOR target".into()),
    }
}

fn find_member_target(service: Service<'_>, structure_target: Target, member_name: &str) -> TestResult<Option<Target>> {
    match structure_target {
        Target::Shape(index) => {
            let shape = service.shape(index)?.ok_or("modeled CBOR shape absent")?;
            let kind = shape.kind()?;
            if kind != "structure" {
                return Ok(None);
            }
            for member in shape.members().map_err(|e| format!("members error: {e}"))? {
                let member = member.map_err(|e| format!("member error: {e}"))?;
                if member.name().map_err(|e| format!("member name error: {e}"))? == member_name {
                    return Ok(Some(member.target));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn encode_number(encoder: &mut minicbor::Encoder<Vec<u8>>, n: &serde_json::Number, timestamp: bool) -> TestResult {
    if timestamp {
        let tag = minicbor::data::Tag::from(minicbor::data::IanaTag::Timestamp);
        encoder.tag(tag).map_err(|e| format!("CBOR tag error: {e}"))?;
        if let Some(i) = n.as_i64() {
            encoder.i64(i).map_err(|e| format!("CBOR i64 error: {e}"))?;
        } else if let Some(f) = n.as_f64() {
            encoder.f64(f).map_err(|e| format!("CBOR f64 error: {e}"))?;
        } else {
            return Err("unsupported timestamp number format".into());
        }
    } else if let Some(i) = n.as_i64() {
        encoder.i64(i).map_err(|e| format!("CBOR i64 error: {e}"))?;
    } else if let Some(u) = n.as_u64() {
        encoder.u64(u).map_err(|e| format!("CBOR u64 error: {e}"))?;
    } else if let Some(f) = n.as_f64() {
        encoder.f64(f).map_err(|e| format!("CBOR f64 error: {e}"))?;
    } else {
        return Err("unsupported JSON number".into());
    }
    Ok(())
}

fn encode_json_with_schema(
    encoder: &mut minicbor::Encoder<Vec<u8>>,
    value: &Json,
    service: Service<'_>,
    target: Option<Target>,
) -> TestResult {
    let kind = target.map(|target| resolve_target_kind(service, target)).transpose()?;

    match value {
        Json::Null => {
            encoder.null().map_err(|e| format!("CBOR null error: {e}"))?;
        }
        Json::Bool(b) => {
            encoder.bool(*b).map_err(|e| format!("CBOR bool error: {e}"))?;
        }
        Json::Number(n) => encode_number(encoder, n, kind == Some("timestamp"))?,
        Json::String(s) => {
            encoder.str(s).map_err(|e| format!("CBOR str error: {e}"))?;
        }
        Json::Array(arr) => {
            let element_target = match (target, kind) {
                (Some(Target::Shape(index)), Some("list")) => {
                    let shape = service.shape(index)?.ok_or("modeled CBOR list shape absent")?;
                    shape.element()?
                }
                _ => None,
            };

            encoder.array(u64::try_from(arr.len()).map_err(|_| "array length exceeds u64")?).map_err(|e| format!("CBOR array error: {e}"))?;
            for item in arr {
                encode_json_with_schema(encoder, item, service, element_target)?;
            }
        }
        Json::Object(map) => {
            encoder.map(u64::try_from(map.len()).map_err(|_| "map length exceeds u64")?).map_err(|e| format!("CBOR map error: {e}"))?;
            for (key, val) in map {
                encoder.str(key).map_err(|e| format!("CBOR key error: {e}"))?;
                let member_target = target.map(|target| find_member_target(service, target, key))
                    .transpose()?.flatten();
                encode_json_with_schema(encoder, val, service, member_target)?;
            }
        }
    }
    Ok(())
}

pub(super) fn decode_rpc_body(body: &[u8]) -> TestResult<serde_json::Map<String, Json>> {
    let mut decoder = minicbor::Decoder::new(body);
    let mut fields = serde_json::Map::new();
    if let Some(count) = decoder.map()? {
        for _ in 0..count {
            decode_rpc_field(&mut decoder, &mut fields)?;
        }
    } else {
        loop {
            if decoder.datatype()? == minicbor::data::Type::Break {
                decoder.skip()?;
                break;
            }
            decode_rpc_field(&mut decoder, &mut fields)?;
        }
    }
    Ok(fields)
}

fn decode_string_array(decoder: &mut minicbor::Decoder<'_>) -> TestResult<Json> {
    use minicbor::data::Type;
    let mut items = Vec::new();
    if let Some(count) = decoder.array()? {
        for _ in 0..count {
            items.push(decode_string_item(decoder)?);
        }
    } else {
        loop {
            if decoder.datatype()? == Type::Break {
                decoder.skip()?;
                break;
            }
            items.push(decode_string_item(decoder)?);
        }
    }
    Ok(Json::Array(items))
}

fn decode_string_item(decoder: &mut minicbor::Decoder<'_>) -> TestResult<Json> {
    if decoder.datatype()? != minicbor::data::Type::String {
        return Err("unsupported array element type in RPC fixture input".into());
    }
    Ok(Json::String(decoder.str()?.to_owned()))
}

fn decode_rpc_field(decoder: &mut minicbor::Decoder<'_>, fields: &mut serde_json::Map<String, Json>) -> TestResult {
    use minicbor::data::Type;
    let name = decoder.str()?.to_owned();
    if fields.contains_key(&name) {
        return Err(format!("duplicate native RPC input field: {name}").into());
    }
    let value = match decoder.datatype()? {
        Type::String => Json::String(decoder.str()?.to_owned()),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => Json::from(decoder.u64()?),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 => Json::from(decoder.i64()?),
        Type::Array | Type::ArrayIndef => decode_string_array(decoder)?,
        _ => return Err(format!("unsupported native RPC fixture input value for field {name}").into()),
    };
    fields.insert(name, value);
    Ok(())
}
