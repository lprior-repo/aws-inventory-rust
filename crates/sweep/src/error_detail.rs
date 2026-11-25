use crate::SweepError;
use inventory_engine::limiter::ApiError;

pub(crate) const MAX_DETAIL_BYTES: usize = inventory_engine::inventory::MAX_OUTCOME_DETAIL_BYTES;

pub(crate) fn detail(error: &SweepError, output_shape: Option<&str>) -> String {
    let class = class(error);
    let message = error.to_string();
    let mut text = match (class, output_shape) {
        ("decode", Some(shape)) => format!("decode: {shape}: {message}"),
        (_, _) => format!("{class}: {message}"),
    };
    if text.len() > MAX_DETAIL_BYTES {
        let mut end = MAX_DETAIL_BYTES;
        while !text.is_char_boundary(end) {
            end = end.saturating_sub(1);
        }
        text.truncate(end);
    }
    text
}

fn class(error: &SweepError) -> &'static str {
    match error {
        SweepError::Decode(_) => "decode",
        SweepError::Encode(_) | SweepError::Signing(_) => "protocol",
        SweepError::Transport(_)
        | SweepError::Api(ApiError::Timeout)
        | SweepError::Api(ApiError::Transient(_)) => "transport",
        SweepError::Api(ApiError::Denied(_)) => "denied",
        SweepError::Api(ApiError::UnsupportedType(_)) => "unsupported",
        SweepError::Api(ApiError::Throttled(_)) => "throttled",
        SweepError::Api(ApiError::RetryAfter { kind, .. }) => match kind {
            inventory_engine::limiter::RetryKind::Throttled => "throttled",
            _ => "transport",
        },
        SweepError::Store(_) => "store",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_engine::inventory::StoreError;

    #[test]
    fn given_decode_error_when_detail_then_class_is_decode() {
        let error = SweepError::Decode(aws_wire::response::DecodeError::Invalid("bad"));
        let detail = detail(&error, None);
        assert!(detail.starts_with("decode: "));
    }

    #[test]
    fn given_decode_error_with_shape_when_detail_then_shape_is_included() {
        let error = SweepError::Decode(aws_wire::response::DecodeError::Invalid("bad"));
        let detail = detail(&error, Some("com.amazonaws.ec2#DescribeVpcsResult"));
        assert!(detail.starts_with("decode: com.amazonaws.ec2#DescribeVpcsResult: "));
    }

    #[test]
    fn given_encode_error_when_detail_then_class_is_protocol() {
        let error = SweepError::Encode(aws_wire::request::EncodeError::InvalidEndpoint);
        let detail = detail(&error, None);
        assert!(detail.starts_with("protocol: "));
    }

    #[test]
    fn given_signing_error_when_detail_then_class_is_protocol() {
        let error = SweepError::Signing(aws_wire::sign::Error::Signing);
        let detail = detail(&error, None);
        assert!(detail.starts_with("protocol: "));
    }

    #[test]
    fn given_transport_error_when_detail_then_class_is_transport() {
        let error = SweepError::Transport(aws_wire::transport::TransportError::Timeout);
        let detail = detail(&error, None);
        assert!(detail.starts_with("transport: "));
    }

    #[test]
    fn given_api_timeout_when_detail_then_class_is_transport() {
        let error = SweepError::Api(ApiError::Timeout);
        let detail = detail(&error, None);
        assert!(detail.starts_with("transport: "));
    }

    #[test]
    fn given_api_transient_when_detail_then_class_is_transport() {
        let error = SweepError::Api(ApiError::Transient("temp".to_owned()));
        let detail = detail(&error, None);
        assert!(detail.starts_with("transport: "));
    }

    #[test]
    fn given_api_denied_when_detail_then_class_is_denied() {
        let error = SweepError::Api(ApiError::Denied("no".to_owned()));
        let detail = detail(&error, None);
        assert!(detail.starts_with("denied: "));
    }

    #[test]
    fn given_api_unsupported_type_when_detail_then_class_is_unsupported() {
        let error = SweepError::Api(ApiError::UnsupportedType("type".to_owned()));
        let detail = detail(&error, None);
        assert!(detail.starts_with("unsupported: "));
    }

    #[test]
    fn given_api_throttled_when_detail_then_class_is_throttled() {
        let error = SweepError::Api(ApiError::Throttled("slow".to_owned()));
        let detail = detail(&error, None);
        assert!(detail.starts_with("throttled: "));
    }

    #[test]
    fn given_retry_after_throttled_when_detail_then_class_is_throttled() {
        let error = SweepError::Api(ApiError::RetryAfter {
            kind: inventory_engine::limiter::RetryKind::Throttled,
            message: "slow".to_owned(),
            retry_after: inventory_engine::limiter::RetryAfter::Invalid,
        });
        let detail = detail(&error, None);
        assert!(detail.starts_with("throttled: "));
    }

    #[test]
    fn given_retry_after_transient_when_detail_then_class_is_transport() {
        let error = SweepError::Api(ApiError::RetryAfter {
            kind: inventory_engine::limiter::RetryKind::Transient,
            message: "temp".to_owned(),
            retry_after: inventory_engine::limiter::RetryAfter::Invalid,
        });
        let detail = detail(&error, None);
        assert!(detail.starts_with("transport: "));
    }

    #[test]
    fn given_store_error_when_detail_then_class_is_store() {
        let error = SweepError::Store(StoreError::StaleLease);
        let detail = detail(&error, None);
        assert!(detail.starts_with("store: "));
    }

    #[test]
    fn given_operation_error_when_detail_then_class_is_other() {
        let error = SweepError::Operation("x");
        let detail = detail(&error, None);
        assert!(detail.starts_with("other: "));
    }

    #[test]
    fn given_long_ascii_message_when_detail_then_truncated_to_max_bytes() {
        let error = SweepError::Credentials("a".repeat(5_000));
        let detail = detail(&error, None);
        assert_eq!(detail.len(), MAX_DETAIL_BYTES);
    }

    #[test]
    fn given_long_multibyte_message_when_detail_then_ends_on_char_boundary() {
        let error = SweepError::Credentials("\u{20ac}".repeat(5_000));
        let detail = detail(&error, None);
        // A three-byte character can leave at most two bytes unused.
        assert!(detail.len() <= MAX_DETAIL_BYTES);
        assert!(detail.len() >= 1_022);
        let last = detail.chars().last();
        assert_eq!(last, Some('\u{20ac}'));
    }

    #[test]
    fn given_short_message_when_detail_then_not_padded() {
        let error = SweepError::Operation("short");
        let detail = detail(&error, None);
        assert_eq!(detail, "other: sweep operation failed: short");
    }
}
