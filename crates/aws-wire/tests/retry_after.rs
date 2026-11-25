#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;

use aws_wire::request::Protocol;
use aws_wire::response::errors::{ErrorClass, ServiceError};
use aws_wire::response::{DecodeLimits, ResponseContext};
use http::{StatusCode, header};
use inventory_core::catalog::Catalog;
use std::time::Duration;

use inventory_engine::limiter::{ApiError, RetryAfter, RetryKind};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ERROR_MODEL: &str = r##"{
"smithy": "2.0",
"shapes": {
  "n#InventoryTest": {
    "type": "service",
    "version": "1",
    "operations": [{"target": "n#ListItems"}],
    "traits": {
      "aws.api#service": {"sdkId": "InventoryTest", "endpointPrefix": "test"},
      "aws.protocols#awsJson1_1": {}
    }
  },
  "n#ListItems": {
    "type": "operation",
    "errors": [{"target": "n#ThrottlingException"}, {"target": "n#RequestTimeout"}, {"target": "n#ResourceNotFoundException"}],
    "output": {"target": "n#Output"}
  },
  "n#ThrottlingException": {
    "type": "structure",
    "traits": {
      "aws.api#throttle": {},
      "smithy.api#error": "client"
    }
  },
  "n#RequestTimeout": {
    "type": "structure",
    "traits": {
      "aws.api#throttle": {},
      "smithy.api#error": "client"
    }
  },
  "n#ResourceNotFoundException": {
    "type": "structure",
    "traits": {
      "smithy.api#error": "client"
    }
  },
  "n#Output": {
    "type": "structure",
    "members": {
      "Items": {"target": "n#ItemsList"}
    }
  },
  "n#ItemsList": {
    "type": "list",
    "member": {"target": "smithy.api#String"}
  }
}}"##;

fn make_catalog() -> Result<
    (
        Catalog<'static>,
        inventory_core::catalog::Service<'static>,
        inventory_core::catalog::Operation<'static>,
    ),
    Box<dyn std::error::Error>,
> {
    let model = serde_json::from_str(ERROR_MODEL)?;
    let bytes = fixture_catalog::compile_service(&model, "n#InventoryTest")?;
    let bytes_static = Box::leak(bytes.into_boxed_slice());
    let catalog = Catalog::open(bytes_static)?;
    let service = catalog.service("InventoryTest")?.ok_or("service missing")?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    Ok((catalog, service, operation))
}

#[test]
fn service_error_carries_retry_after_from_header() -> TestResult {
    let (_catalog, service, operation) = make_catalog()?;
    let headers = {
        let mut m = http::HeaderMap::new();
        m.insert(header::RETRY_AFTER, header::HeaderValue::from_static("3"));
        m
    };
    let context = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"",
        headers: &headers,
        status: StatusCode::TOO_MANY_REQUESTS,
        limits: DecodeLimits::default(),
    };
    let error = aws_wire::response::errors::decode(context)?;
    assert_eq!(error.class, ErrorClass::Throttle);
    assert_eq!(error.retry_after, Some(Duration::from_secs(3)));
    Ok(())
}

#[test]
fn service_error_without_retry_after_header_is_none() -> TestResult {
    let (_catalog, service, operation) = make_catalog()?;
    let headers = http::HeaderMap::new();
    let context = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"",
        headers: &headers,
        status: StatusCode::TOO_MANY_REQUESTS,
        limits: DecodeLimits::default(),
    };
    let error = aws_wire::response::errors::decode(context)?;
    assert_eq!(error.class, ErrorClass::Throttle);
    assert_eq!(error.retry_after, None);
    Ok(())
}

#[test]
fn oversized_retry_after_is_clamped_to_the_cap() -> TestResult {
    let (_catalog, service, operation) = make_catalog()?;
    let headers = {
        let mut m = http::HeaderMap::new();
        m.insert(
            header::RETRY_AFTER,
            header::HeaderValue::from_static("9999999999999999999999999999"),
        );
        m
    };
    let context = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"",
        headers: &headers,
        status: StatusCode::TOO_MANY_REQUESTS,
        limits: DecodeLimits::default(),
    };
    let error = aws_wire::response::errors::decode(context)?;
    assert_eq!(error.class, ErrorClass::Throttle);
    assert_eq!(error.retry_after, Some(std::time::Duration::from_secs(300)));
    Ok(())
}

#[test]
fn non_numeric_retry_after_is_ignored_not_panic() -> TestResult {
    let (_catalog, service, operation) = make_catalog()?;
    let headers = {
        let mut m = http::HeaderMap::new();
        m.insert(
            header::RETRY_AFTER,
            header::HeaderValue::from_static("not-a-number"),
        );
        m
    };
    let context = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"",
        headers: &headers,
        status: StatusCode::TOO_MANY_REQUESTS,
        limits: DecodeLimits::default(),
    };
    let error = aws_wire::response::errors::decode(context)?;
    assert_eq!(error.class, ErrorClass::Throttle);
    assert_eq!(error.retry_after, None);
    Ok(())
}

#[test]
fn throttle_error_class_with_retry_after_propagates_to_api_error() {
    let error = ServiceError {
        class: ErrorClass::Throttle,
        code: Some("ProvisionedThroughputExceededException".to_owned()),
        retry_after: Some(Duration::from_secs(2)),
    };
    let message = error.to_string();
    let api_error = match error.class {
        ErrorClass::Throttle => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Throttled,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Throttled(message)
            }
        }
        ErrorClass::Retryable | ErrorClass::ExpiredToken | ErrorClass::ExpiredSignature => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Transient,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Transient(message)
            }
        }
        ErrorClass::AccessDenied => ApiError::Denied(message),
        ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        ErrorClass::Fatal => ApiError::Permanent(message),
    };
    assert!(matches!(api_error, ApiError::RetryAfter { .. }));
}

#[test]
fn transient_error_class_with_retry_after_propagates_to_api_error() {
    let error = ServiceError {
        class: ErrorClass::Retryable,
        code: Some("RequestTimeout".to_owned()),
        retry_after: Some(Duration::from_secs(1)),
    };
    let message = error.to_string();
    let api_error = match error.class {
        ErrorClass::Throttle => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Throttled,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Throttled(message)
            }
        }
        ErrorClass::Retryable | ErrorClass::ExpiredToken | ErrorClass::ExpiredSignature => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Transient,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Transient(message)
            }
        }
        ErrorClass::AccessDenied => ApiError::Denied(message),
        ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        ErrorClass::Fatal => ApiError::Permanent(message),
    };
    assert!(matches!(api_error, ApiError::RetryAfter { .. }));
}

#[test]
fn fatal_error_ignores_retry_after_even_when_present() {
    let error = ServiceError {
        class: ErrorClass::Fatal,
        code: Some("ResourceNotFoundException".to_owned()),
        retry_after: Some(Duration::from_secs(5)),
    };
    let message = error.to_string();
    let api_error = match error.class {
        ErrorClass::Throttle => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Throttled,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Throttled(message)
            }
        }
        ErrorClass::Retryable | ErrorClass::ExpiredToken | ErrorClass::ExpiredSignature => {
            if let Some(duration) = error.retry_after {
                ApiError::RetryAfter {
                    kind: RetryKind::Transient,
                    message,
                    retry_after: RetryAfter::Delay(duration),
                }
            } else {
                ApiError::Transient(message)
            }
        }
        ErrorClass::AccessDenied => ApiError::Denied(message),
        ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        ErrorClass::Fatal => ApiError::Permanent(message),
    };
    assert!(matches!(api_error, ApiError::Permanent(_)));
}
