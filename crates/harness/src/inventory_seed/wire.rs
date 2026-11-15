use super::{
    config::{MutationPermit, SeedError},
    model::SeedOperation,
};
use aws_wire::sign::Region as SigningRegion;
use aws_wire::{
    request::{CompressionPolicy, RequestContext, RequestLimits},
    response::{DecodeLimits, ResponseContext},
    sign::{Body, PathMode, PayloadHeader, Request as SignRequest, Scope, SigningName},
    transport::Client,
};
use bytes::Bytes;
use http::Request;
use inventory_core::record::{FieldId, Key, MapRef, Record, Value};
use std::{collections::BTreeMap, time::SystemTime};

#[derive(Clone, Copy)]
pub(super) struct RequestSpec<'a> {
    pub operation: SeedOperation<'a>,
    pub input: MapRef<'a>,
    pub endpoint: &'a str,
    pub region: &'a str,
}

pub(super) fn input_record(
    fields: impl IntoIterator<Item = (Vec<FieldId>, Value)>,
) -> Result<Record, SeedError> {
    let mut values = BTreeMap::new();
    for (path, value) in fields {
        insert_path(&mut values, &path, value)?;
    }
    Ok(Record::encode(&Value::Map(values))?)
}

fn insert_path(
    map: &mut BTreeMap<Key, Value>,
    path: &[FieldId],
    value: Value,
) -> Result<(), SeedError> {
    let Some((field, rest)) = path.split_first() else {
        return Err(SeedError::Invalid("seed input field path is empty"));
    };
    let key = Key::Field(*field);
    if rest.is_empty() {
        return map
            .insert(key, value)
            .is_none()
            .then_some(())
            .ok_or(SeedError::Invalid("seed input field path is duplicated"));
    }
    let nested = map
        .entry(key)
        .or_insert_with(|| Value::Map(BTreeMap::new()));
    match nested {
        Value::Map(nested) => insert_path(nested, rest, value),
        _ => Err(SeedError::Invalid("seed input field paths conflict")),
    }
}

pub(super) async fn call_mutation(
    _permit: &MutationPermit,
    client: &Client,
    credentials: &aws_wire::sign::Credentials,
    spec: RequestSpec<'_>,
) -> Result<Record, SeedError> {
    call(client, credentials, spec).await
}

pub(super) async fn call(
    client: &Client,
    credentials: &aws_wire::sign::Credentials,
    spec: RequestSpec<'_>,
) -> Result<Record, SeedError> {
    let request = build_request(credentials, spec)?;
    let response = client.send(request).await?;
    let descriptor = spec.operation;
    Ok(aws_wire::response::decode(ResponseContext {
        service: descriptor.service,
        operation: descriptor.operation,
        protocol: descriptor.protocol,
        body: response.body.as_ref(),
        headers: &response.headers,
        status: response.status,
        limits: DecodeLimits::default(),
    })?)
}

fn build_request(
    credentials: &aws_wire::sign::Credentials,
    spec: RequestSpec<'_>,
) -> Result<Request<Bytes>, SeedError> {
    let descriptor = spec.operation;
    let request = encode_request(spec)?;
    sign_request(credentials, request, descriptor.service.signing_name()?, spec.region)
}

fn encode_request(spec: RequestSpec<'_>) -> Result<Request<Bytes>, SeedError> {
    let descriptor = spec.operation;
    let context = RequestContext {
        service: descriptor.service,
        operation: descriptor.operation,
        protocol: descriptor.protocol,
        input: spec.input,
        endpoint: spec.endpoint,
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    };
    Ok(match descriptor.protocol {
        aws_wire::request::Protocol::AwsQuery | aws_wire::request::Protocol::Ec2Query =>
            aws_wire::request::query::encode(context)?,
        aws_wire::request::Protocol::RestJson1 | aws_wire::request::Protocol::RestXml =>
            aws_wire::request::rest::encode(context)?,
        _ => aws_wire::request::body::encode(context)?,
    })
}

fn sign_request(
    credentials: &aws_wire::sign::Credentials,
    mut request: Request<Bytes>,
    name: Option<&str>,
    region: &str,
) -> Result<Request<Bytes>, SeedError> {
    let name = name.ok_or(SeedError::Invalid("modeled service has no signing name"))?;
    let uri = request.uri().to_string();
    let scope = Scope {
        region: SigningRegion::new(region)?,
        name: SigningName::new(name)?,
    };
    let signed = SignRequest {
        method: request.method(),
        uri: &uri,
        headers: request.headers(),
        body: Body::Bytes(request.body().as_ref()),
        payload_header: PayloadHeader::Include,
    };
    let now = SystemTime::now();
    let headers = aws_wire::sign::sign(credentials, &scope, now, signed, PathMode::Standard)?;
    request.headers_mut().extend(headers);
    Ok(request)
}
