use std::time::Duration;

use super::ddl_bounded;
use crate::HarnessError;

#[tokio::test(start_paused = true)]
async fn given_pending_future_when_ddl_bounded_then_exceeds_65_seconds()
-> Result<(), Box<dyn std::error::Error>> {
    let start = tokio::time::Instant::now();
    let future = std::future::pending::<Result<(), HarnessError>>();
    let result = ddl_bounded(future).await;
    let elapsed = start.elapsed();

    match result {
        Err(error) => {
            let message = error.to_string();
            assert!(
                message.contains("exceeded 65 seconds"),
                "error should mention timeout: {message}"
            );
        }
        Ok(_) => {
            return Err("expected an error from timed out future".into());
        }
    }

    assert_eq!(
        elapsed,
        Duration::from_secs(65),
        "timeout should trigger at exactly 65 seconds"
    );

    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_64_second_future_when_ddl_bounded_then_returns_ok()
-> Result<(), Box<dyn std::error::Error>> {
    let future = async {
        tokio::time::sleep(Duration::from_secs(64)).await;
        Ok(7_u8)
    };
    let result = ddl_bounded(future).await;

    match result {
        Ok(value) => assert_eq!(value, 7, "value should be 7"),
        Err(error) => {
            return Err(
                format!("a 64-second statement must finish inside the bound: {error}").into(),
            );
        }
    }

    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_immediate_error_when_ddl_bounded_then_returns_inner_error()
-> Result<(), Box<dyn std::error::Error>> {
    let future = async { Err(HarnessError::invalid("inner")) };
    let result: Result<(), HarnessError> = ddl_bounded(future).await;

    match result {
        Err(error) => {
            let message = error.to_string();
            assert!(
                message.contains("inner"),
                "error should contain inner message: {message}"
            );
            assert!(
                !message.contains("65 seconds"),
                "immediate error should not mention timeout: {message}"
            );
        }
        Ok(_) => {
            return Err("expected an error from future".into());
        }
    }

    Ok(())
}
