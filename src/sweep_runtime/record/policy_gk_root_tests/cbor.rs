
use super::TestResult;
use inventory_core::catalog::runtime::{CallablePlan, RuntimeProtocol};
use minicbor::{Decoder, Encoder, data::Type};
use serde_json::{Map, Number, Value as Json};

const TIMESTAMP_MEMBERS: &[&str] = &["CreationTime"];

pub(in crate::sweep_runtime::record) fn response(plan: &CallablePlan<'_>, body: &Json) -> TestResult<Vec<u8>> {
    if plan.protocol() == RuntimeProtocol::RpcV2Cbor {
        let mut encoder = Encoder::new(Vec::new());
        encode(&mut encoder, None, body)?;
        Ok(encoder.into_writer())
    } else {
        Ok(serde_json::to_vec(body)?)
    }
}

pub(in crate::sweep_runtime::record) fn request(wire: &http::Request<bytes::Bytes>, service: &str, operation: &str) -> TestResult<Json> {
    assert_eq!(wire.method(), http::Method::POST);
    assert_eq!(wire.uri().path(), format!("/service/{service}/operation/{operation}"));
    assert_eq!(wire.uri().query(), None);
    assert!(wire.headers().get("x-amz-target").is_none(), "RPC v2 CBOR must not use x-amz-target");
    assert_eq!(wire.headers().get("content-type").map(http::HeaderValue::to_str).transpose()?, Some("application/cbor"));
    assert_eq!(wire.headers().get("smithy-protocol").map(http::HeaderValue::to_str).transpose()?, Some("rpc-v2-cbor"));
    if wire.body().is_empty() {
        return Ok(Json::Object(Map::new()));
    }
    let mut decoder = Decoder::new(wire.body());
    let value = decode(&mut decoder)?;
    assert_eq!(decoder.position(), wire.body().len(), "trailing CBOR data in {operation} request");
    Ok(value)
}

fn encode(encoder: &mut Encoder<Vec<u8>>, member: Option<&str>, value: &Json) -> TestResult {
    match value {
        Json::Null => { encoder.null()?; }
        Json::Bool(flag) => { encoder.bool(*flag)?; }
        Json::Number(number) => {
            if member.is_some_and(|name| TIMESTAMP_MEMBERS.contains(&name)) { encoder.tag(minicbor::data::Tag::new(1))?; }
            match (number.as_i64(), number.as_u64(), number.as_f64()) {
                (Some(signed), _, _) => { encoder.i64(signed)?; }
                (None, Some(unsigned), _) => { encoder.u64(unsigned)?; }
                (None, None, Some(float)) => { encoder.f64(float)?; }
                _ => return Err("unrepresentable JSON number".into()),
            }
        }
        Json::String(text) => { encoder.str(text)?; }
        Json::Array(items) => {
            encoder.array(u64::try_from(items.len())?)?;
            for item in items { encode(encoder, None, item)?; }
        }
        Json::Object(members) => {
            encoder.map(u64::try_from(members.len())?)?;
            for (name, item) in members {
                encoder.str(name)?;
                encode(encoder, Some(name), item)?;
            }
        }
    }
    Ok(())
}

fn decode(decoder: &mut Decoder<'_>) -> TestResult<Json> {
    Ok(match decoder.datatype()? {
        Type::Null => { decoder.null()?; Json::Null }
        Type::Bool => Json::Bool(decoder.bool()?),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => Json::from(decoder.u64()?),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => Json::from(decoder.i64()?),
        Type::F16 | Type::F32 | Type::F64 => Json::Number(Number::from_f64(decoder.f64()?).ok_or("non-finite CBOR float")?),
        Type::String => Json::String(decoder.str()?.to_owned()),
        Type::StringIndef => Json::String(decoder.str_iter()?.collect::<Result<String, _>>()?),
        Type::Tag => { decoder.tag()?; decode(decoder)? }
        Type::Array | Type::ArrayIndef => {
            let mut items = Vec::new();
            let mut remaining = decoder.array()?;
            loop {
                match remaining {
                    Some(0) => break,
                    Some(left) => remaining = Some(left - 1),
                    None if decoder.datatype()? == Type::Break => { decoder.skip()?; break; }
                    None => {}
                }
                items.push(decode(decoder)?);
            }
            Json::Array(items)
        }
        Type::Map | Type::MapIndef => {
            let mut members = Map::new();
            let mut remaining = decoder.map()?;
            loop {
                match remaining {
                    Some(0) => break,
                    Some(left) => remaining = Some(left - 1),
                    None if decoder.datatype()? == Type::Break => { decoder.skip()?; break; }
                    None => {}
                }
                let name = decoder.str()?.to_owned();
                let value = decode(decoder)?;
                assert!(members.insert(name.clone(), value).is_none(), "duplicate CBOR member {name}");
            }
            Json::Object(members)
        }
        other => return Err(format!("unsupported CBOR type {other:?} in fixture").into()),
    })
}
