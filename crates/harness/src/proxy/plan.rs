use super::{MAX_SCOPE_NAME, ProxyError, validated::ValidatedConfig};
use std::time::Duration;

/// Nonempty bounded ASCII scope component, safe in protocol diagnostics.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeName(String);
impl ScopeName {
    /// Parse 1..=128 ASCII letters, digits, underscores, dots or hyphens.
    pub fn parse(value: &str) -> Result<Self, ProxyError> {
        if value.is_empty()
            || value.len() > MAX_SCOPE_NAME
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        {
            return Err(ProxyError::Configuration("scope component"));
        }
        Ok(Self(value.to_owned()))
    }
    /// Borrow the validated component.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Independent scope identity. Signed tuples sort lexically before Unsigned.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Scope {
    /// Parsed SigV4 account/access-key identity, region and signing service.
    Signed {
        /// Mapped account, or access key when no mapping exists.
        account: ScopeName,
        /// SigV4 region.
        region: ScopeName,
        /// SigV4 signing service.
        service: ScopeName,
    },
    /// One fallback identity for missing or unparseable credentials.
    Unsigned,
}
impl Scope {
    /// Parse three bounded signed-scope components; no spelling denotes Unsigned.
    pub fn new(account: &str, region: &str, service: &str) -> Result<Self, ProxyError> {
        Ok(Self::Signed {
            account: ScopeName::parse(account)?,
            region: ScopeName::parse(region)?,
            service: ScopeName::parse(service)?,
        })
    }
}

/// Faults selectable by a probability rule or an exact ordinal schedule.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fault {
    /// AWS throttling error.
    Throttle,
    /// HTTP 500 InternalFailure.
    InternalFailure,
    /// HTTP 503 ServiceUnavailable.
    ServiceUnavailable,
    /// Consume request and send nothing until peer closes or shutdown.
    Stall,
    /// Abortive TCP close after consuming request.
    Reset,
    /// Strict prefix of the upstream response body, then close.
    Truncation,
    /// Same-length invalid JSON/XML/CBOR payload.
    MalformedBody,
    /// AWS authorization denial.
    AccessDenied,
    /// AWS pagination token expiration.
    ExpiredPaginationToken,
    /// AWS credential expiration.
    ExpiredCredentials,
    /// AWS signature expiration.
    ExpiredSignature,
    /// Drop trigger and refuse new connects for the configured window.
    RefusedConnection,
}

/// Probability in integer parts per million, inclusive of zero and certainty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Probability(u32);
impl Probability {
    /// Reject values above one million.
    pub fn new(parts_per_million: u32) -> Result<Self, ProxyError> {
        if parts_per_million > 1_000_000 {
            return Err(ProxyError::Configuration("probability exceeds one million"));
        }
        Ok(Self(parts_per_million))
    }
}

/// Replayable decision; entropy also selects truncation fraction and corrupt bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decision {
    /// None means ordinary forwarding.
    pub fault: Option<Fault>,
    /// Seed-derived word, independent of clocks and task scheduling.
    pub entropy: u64,
    /// Fixed added response delay from the first matching latency rule.
    pub latency: Duration,
}

/// SplitMix64 finalizer and Weyl increment, using the published constants.
/// Arithmetic deliberately wraps modulo 2^64, never ambient randomness.
fn splitmix(value: u64) -> u64 {
    let z = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn scope_word(seed: u64, scope: &Scope) -> u64 {
    const SIGNED_DOMAIN: u64 = 0x7369_676e_6564_0001;
    let (word, names) = match scope {
        Scope::Signed {
            account,
            region,
            service,
        } => (
            seed ^ SIGNED_DOMAIN,
            [account.as_str(), region.as_str(), service.as_str()],
        ),
        // Preserve the published unsigned replay vectors, not a signed identity.
        Scope::Unsigned => (seed, ["unsigned"; 3]),
    };
    names.into_iter().fold(word, |word, name| {
        name.bytes()
            .chain([0])
            .fold(word, |w, byte| splitmix(w ^ u64::from(byte)))
    })
}

/// Pure plan evaluation. Exact schedules win, then the first matching successful
/// probability rule. Each rule has an independent SplitMix64 word; order is stable.
/// Only the frozen startup-validated representation can enter evaluation.
pub(crate) fn decide(config: &ValidatedConfig, scope: &Scope, ordinal: u64) -> Decision {
    let entropy = splitmix(scope_word(config.seed(), scope) ^ splitmix(ordinal));
    let scheduled = config
        .schedule()
        .iter()
        .find(|s| s.scope == *scope && s.ordinal == ordinal);
    let fault = scheduled
        .map(|s| s.fault)
        .or_else(|| random_fault(config, scope, entropy));
    let latency = config.delay(scope);
    Decision {
        fault,
        entropy,
        latency,
    }
}

pub(crate) fn signed_scope(authorization: Option<&[u8]>, config: &ValidatedConfig) -> Scope {
    match authorization
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|header| parse_scope(header, config))
    {
        Some(scope) => scope,
        None => Scope::Unsigned,
    }
}

fn parse_scope(header: &str, config: &ValidatedConfig) -> Option<Scope> {
    let credential = credential(header)?;
    let mut parts = credential.split('/');
    let key = ScopeName::parse(parts.next()?).ok()?;
    let date = parts.next()?;
    if date.len() != 8 || !date.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let region = ScopeName::parse(parts.next()?).ok()?;
    let service = ScopeName::parse(parts.next()?).ok()?;
    if parts.next()? != "aws4_request" || parts.next().is_some() {
        return None;
    }
    let account = mapped_account(config, &key);
    Some(Scope::Signed {
        account,
        region,
        service,
    })
}

fn random_fault(config: &ValidatedConfig, scope: &Scope, entropy: u64) -> Option<Fault> {
    config
        .rules()
        .iter()
        .zip(0_u64..)
        .find(|(rule, index_word)| {
            rule.filter.matches(scope)
                && splitmix(entropy ^ splitmix(*index_word)) % 1_000_000
                    < u64::from(rule.probability.0)
        })
        .map(|(rule, _)| rule.fault)
}

fn mapped_account(config: &ValidatedConfig, key: &ScopeName) -> ScopeName {
    config
        .accounts()
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map_or_else(|| key.clone(), |(_, account)| account.clone())
}

fn credential(header: &str) -> Option<&str> {
    header
        .strip_prefix("AWS4-HMAC-SHA256 ")?
        .split(',')
        .find_map(|part| part.trim().strip_prefix("Credential="))
}
