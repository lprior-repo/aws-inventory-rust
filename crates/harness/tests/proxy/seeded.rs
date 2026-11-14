use super::support::{OK, Server, TestResult, UNSIGNED, exchange, field, split};
use inventory_harness::proxy::{Fault, Proxy};

struct Case {
    seed: u64,
    entropy: u64,
    prefix: usize,
    byte: u8,
}
// Independent SplitMix64 evaluation of unsigned\0 repeated three times, then
// ordinal zero: prefix=floor(15*(1+entropy%3)/4), byte=entropy's low byte.
const CASES: [Case; 3] = [
    Case {
        seed: 7,
        entropy: 0xf54c_f35a_9875_11c3,
        prefix: 7,
        byte: 0xc3,
    },
    Case {
        seed: 11,
        entropy: 0x94a1_790f_451d_7cdc,
        prefix: 11,
        byte: 0xdc,
    },
    Case {
        seed: 13,
        entropy: 0x02df_74b3_092b_5d0d,
        prefix: 3,
        byte: 0x0d,
    },
];

async fn sample(
    server: &Server,
    case: &Case,
    fault: Fault,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut config = server.fault(fault);
    config.seed = case.seed;
    let proxy = Proxy::start(config).await?;
    let bytes = exchange(proxy.address(), UNSIGNED).await?;
    assert_eq!(
        proxy
            .snapshot()?
            .decisions
            .first()
            .map(|entry| entry.decision.entropy),
        Some(case.entropy)
    );
    proxy.shutdown().await?;
    Ok(bytes)
}

pub async fn truncation() -> TestResult {
    let server = Server::start(OK).await?;
    for case in &CASES {
        let first = sample(&server, case, Fault::Truncation).await?;
        assert_eq!(first, sample(&server, case, Fault::Truncation).await?);
        assert_eq!(split(&first)?.0, split(OK)?.0);
        let body = split(&first)?.1;
        let expected = split(OK)?.1.get(..case.prefix).ok_or("prefix fixture")?;
        assert_eq!(body, expected);
        assert!(field(&first, "content-length")?.parse::<usize>()? > body.len());
    }
    Ok(())
}

pub async fn malformed() -> TestResult {
    let server = Server::start(OK).await?;
    for case in &CASES {
        let first = sample(&server, case, Fault::MalformedBody).await?;
        assert_eq!(first, sample(&server, case, Fault::MalformedBody).await?);
        check_corruption(case, &first)?;
    }
    Ok(())
}

fn check_corruption(case: &Case, bytes: &[u8]) -> TestResult {
    assert_eq!(split(bytes)?.0, split(OK)?.0);
    let body = split(bytes)?.1;
    let mut expected = vec![case.byte; split(OK)?.1.len()];
    *expected.first_mut().ok_or("empty fixture")? = 0xff;
    assert_eq!(body, expected);
    assert!(serde_json::from_slice::<serde_json::Value>(body).is_err());
    assert!(std::str::from_utf8(body).is_err());
    assert!(minicbor::Decoder::new(body).map().is_err());
    Ok(())
}
