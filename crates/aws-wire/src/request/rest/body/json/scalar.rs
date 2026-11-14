use super::super::super::super::body::json as scalar_json;
use super::super::super::super::common::BoundedBody;
use super::super::super::EncodeError;
use inventory_core::catalog::TimestampFormat;

pub(super) fn write_bytes(body: &mut BoundedBody, bytes: &[u8]) -> Result<(), EncodeError> {
    body.write(b"\"")?;
    scalar_json::base64(body, bytes)?;
    body.write(b"\"")
}

pub(super) fn write_timestamp(
    body: &mut BoundedBody,
    time: inventory_core::record::Time,
    format: Option<TimestampFormat>,
) -> Result<(), EncodeError> {
    let format = format.unwrap_or(TimestampFormat::EpochSeconds);
    let text = super::super::super::timestamp::format(time, Some(format))?;
    if format == TimestampFormat::EpochSeconds {
        body.write(text.as_bytes())
    } else {
        scalar_json::string(body, &text)
    }
}

pub(super) fn write_name(body: &mut BoundedBody, name: &str) -> Result<(), EncodeError> {
    scalar_json::string(body, name)
}
