use super::support::{OK, Server, TestResult, exchange, field, finished, request};
use inventory_harness::proxy::{
    Config, DecisionEntry, Fault, Probability, Proxy, Rule, Scope, ScopeFilter,
};

type Coordinates = (&'static str, &'static str, &'static str);
const ROUNDS: u64 = 12;
const SCOPES: [Coordinates; 3] = [
    ("a", "us-east-1", "s3"),
    ("b", "us-west-2", "ec2"),
    ("c", "eu-west-1", "iam"),
];
pub enum Order {
    Forward,
    Reverse,
}

pub fn config(server: &Server, seed: u64) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = server.config();
    config.seed = seed;
    for fault in [Fault::Throttle, Fault::AccessDenied, Fault::InternalFailure] {
        config.rules.push(Rule {
            filter: ScopeFilter::default(),
            probability: Probability::new(350_000)?,
            fault,
        });
    }
    Ok(config)
}

pub async fn sequence(
    server: &Server,
    seed: u64,
    order: Order,
) -> Result<Vec<DecisionEntry>, Box<dyn std::error::Error>> {
    let proxy = Proxy::start(config(server, seed)?).await?;
    let [a, b, c] = SCOPES;
    let scopes = match order {
        Order::Forward => [a, b, c],
        Order::Reverse => [c, b, a],
    };
    for ordinal in 0..ROUNDS {
        for coordinates in scopes {
            scope_request(&proxy, coordinates, ordinal).await?;
        }
    }
    finished(&proxy, ROUNDS.checked_mul(3).ok_or("round overflow")?).await?;
    let log = proxy.snapshot()?.decisions;
    proxy.shutdown().await?;
    Ok(log)
}

async fn scope_request(proxy: &Proxy, coordinates: Coordinates, ordinal: u64) -> TestResult {
    let (key, region, service) = coordinates;
    let bytes = exchange(proxy.address(), &request(key, region, service)).await?;
    let scope = Scope::new(key, region, service)?;
    let snapshot = proxy.snapshot()?;
    let entry = snapshot
        .decisions
        .iter()
        .find(|entry| entry.scope == scope && entry.ordinal == ordinal);
    observe(entry.ok_or("missing scoped decision")?, &bytes)
}

pub fn observe(entry: &DecisionEntry, bytes: &[u8]) -> TestResult {
    let (status, code) = match entry.decision.fault {
        None => {
            assert_eq!(bytes, OK);
            return Ok(());
        }
        Some(Fault::Throttle) => (400, "ThrottlingException"),
        Some(Fault::AccessDenied) => (403, "AccessDeniedException"),
        Some(Fault::InternalFailure) => (500, "InternalFailure"),
        _ => return Err("unexpected probabilistic fault".into()),
    };
    assert!(bytes.starts_with(format!("HTTP/1.1 {status} ").as_bytes()));
    assert_eq!(field(bytes, "x-amzn-ErrorType")?, code);
    Ok(())
}

pub fn ordinals(entries: &[DecisionEntry]) -> TestResult {
    for (key, region, service) in SCOPES {
        let scope = Scope::new(key, region, service)?;
        let actual: Vec<_> = entries
            .iter()
            .filter(|entry| entry.scope == scope)
            .map(|entry| entry.ordinal)
            .collect();
        assert_eq!(actual, (0..ROUNDS).collect::<Vec<_>>());
    }
    assert!(entries.iter().any(|entry| entry.decision.fault.is_none()));
    assert!(entries.iter().any(|entry| entry.decision.fault.is_some()));
    Ok(())
}

pub async fn verify(server: &Server) -> TestResult {
    let first = sequence(server, 7, Order::Forward).await?;
    let second = sequence(server, 7, Order::Reverse).await?;
    let other = sequence(server, 8, Order::Forward).await?;
    assert_eq!(first, second);
    assert_ne!(
        first
            .iter()
            .map(|entry| entry.decision.fault)
            .collect::<Vec<_>>(),
        other
            .iter()
            .map(|entry| entry.decision.fault)
            .collect::<Vec<_>>()
    );
    ordinals(&first)
}
