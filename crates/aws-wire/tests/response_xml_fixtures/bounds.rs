use aws_wire::response::{DecodeError, DecodeLimits, ResponseContext};

pub(super) fn malformed_xml_is_rejected(context: ResponseContext<'_>) {
    for body in [
        b"<broken>".as_slice(),
        b"<!DOCTYPE root [<!ENTITY x 'y'>]><root>&x;</root>".as_slice(),
        b"<root/><root/>".as_slice(),
    ] {
        assert!(matches!(
            aws_wire::response::decode(ResponseContext { body, ..context }),
            Err(DecodeError::Invalid(_))
        ));
    }
}

pub(super) fn limits_are_enforced(context: ResponseContext<'_>) {
    let body = b"<unknown><inner><deep/></inner></unknown>";
    for (limits, expected) in [
        (
            DecodeLimits::new(1, 65_536, 32).expect("body cap"),
            "response body",
        ),
        (
            DecodeLimits::new(1024, 1, 32).expect("node cap"),
            "response nodes",
        ),
        (
            DecodeLimits::new(1024, 65_536, 1).expect("depth cap"),
            "response depth",
        ),
    ] {
        let result = aws_wire::response::decode(ResponseContext {
            body,
            limits,
            ..context
        });
        assert!(matches!(result, Err(DecodeError::Limit(actual)) if actual == expected));
    }
}
