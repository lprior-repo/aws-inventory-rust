use super::{EncodeError, uri};
use crate::request::common::BoundedBody;

mod fixtures;

#[test]
fn rest_uri_labels_encode_regular_and_greedy_values_exactly() -> Result<(), EncodeError> {
    assert_eq!(uri::encode_label("a b/c", false, 32)?, "a%20b%2Fc");
    assert_eq!(uri::encode_label("a b/c", true, 32)?, "a%20b/c");
    Ok(())
}
#[test]
fn rest_uri_label_encoding_rejects_oversized_output() {
    assert!(matches!(
        uri::encode_label("large", false, 3),
        Err(EncodeError::Limit("URI"))
    ));
}

#[test]
fn bounded_body_rejects_output_past_the_configured_limit() {
    let mut body = BoundedBody::new(4);
    assert!(body.write(b"1234").is_ok());
    assert!(matches!(body.write(b"5"), Err(EncodeError::Limit("body"))));
}
