use super::{
    Decision, Fault, ProxyError, Scope,
    validated::{RatePolicy, ValidatedConfig},
};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    num::NonZeroU32,
};
use tokio::time::Instant;

const TOKEN_UNITS: u128 = 1_000_000_000;
type PlannedDecision = (u64, Decision);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    pub accepted: u64,

    pub requests: u64,

    pub forwarded: u64,

    pub faults: BTreeMap<Fault, u64>,

    pub delayed: u64,

    pub rate_limited: u64,

    pub responses_ready: u64,

    pub responses_written: u64,

    pub finished: u64,

    pub stalled_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionEntry {
    pub scope: Scope,

    pub ordinal: u64,

    pub decision: Decision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub counters: Counters,

    pub decisions: Vec<DecisionEntry>,

    pub dropped: u64,

    pub scopes: usize,
}

struct Bucket {
    ordinal: u64,
    tokens: u128,
    updated: Instant,
}
pub(crate) struct State {
    scopes: HashMap<Scope, Bucket>,
    log: VecDeque<DecisionEntry>,
    counters: Counters,
    dropped: u64,
}
#[derive(Clone)]
pub(crate) struct Shared(Arc<Mutex<State>>);
impl Shared {
    pub(crate) fn new(config: &ValidatedConfig) -> Result<Self, ProxyError> {
        let mut scopes = HashMap::new();
        scopes
            .try_reserve(config.scope_capacity())
            .map_err(|_| ProxyError::Allocation)?;
        let mut log = VecDeque::new();
        log.try_reserve(config.log_capacity())
            .map_err(|_| ProxyError::Allocation)?;
        Ok(Self(Arc::new(Mutex::new(State {
            scopes,
            log,
            counters: Counters::default(),
            dropped: 0,
        }))))
    }
    fn lock(&self) -> Result<MutexGuard<'_, State>, ProxyError> {
        self.0.lock().map_err(|_| ProxyError::Poisoned)
    }
    pub(crate) fn snapshot(&self) -> Result<Snapshot, ProxyError> {
        let state = self.lock()?;
        let mut decisions: Vec<_> = state.log.iter().cloned().collect();
        decisions.sort_unstable_by(|a, b| (&a.scope, a.ordinal).cmp(&(&b.scope, b.ordinal)));
        Ok(Snapshot {
            counters: state.counters.clone(),
            decisions,
            dropped: state.dropped,
            scopes: state.scopes.len(),
        })
    }
    pub(crate) fn accepted(&self) -> Result<(), ProxyError> {
        let mut state = self.lock()?;
        state.counters.accepted = state.counters.accepted.saturating_add(1);
        Ok(())
    }
    pub(crate) fn forwarded(&self) -> Result<(), ProxyError> {
        let mut state = self.lock()?;
        state.counters.forwarded = state.counters.forwarded.saturating_add(1);
        Ok(())
    }
    pub(crate) fn decide(
        &self,
        config: &ValidatedConfig,
        scope: &Scope,
    ) -> Result<PlannedDecision, ProxyError> {
        let mut state = self.lock()?;
        let now = Instant::now();
        state.counters.requests = state.counters.requests.saturating_add(1);
        let (ordinal, decision) = state.next_decision(config, scope, now)?;
        state.record(
            config,
            DecisionEntry {
                scope: scope.clone(),
                ordinal,
                decision,
            },
        );
        Ok((ordinal, decision))
    }
    pub(crate) fn response_ready(&self) -> Result<(), ProxyError> {
        let mut state = self.lock()?;
        state.counters.responses_ready = state.counters.responses_ready.saturating_add(1);
        Ok(())
    }
    pub(crate) fn response_written(&self) -> Result<(), ProxyError> {
        let mut state = self.lock()?;
        state.counters.responses_written = state.counters.responses_written.saturating_add(1);
        Ok(())
    }
    pub(crate) fn finished(&self) -> Result<(), ProxyError> {
        let mut state = self.lock()?;
        state.counters.finished = state.counters.finished.saturating_add(1);
        Ok(())
    }
    pub(crate) fn stalled_bytes(&self, count: usize) -> Result<(), ProxyError> {
        let count = u64::try_from(count).map_err(|_| ProxyError::Limit("stalled byte count"))?;
        let mut state = self.lock()?;
        state.counters.stalled_bytes = state.counters.stalled_bytes.saturating_add(count);
        Ok(())
    }
}
impl State {
    fn next_decision(
        &mut self,
        config: &ValidatedConfig,
        scope: &Scope,
        now: Instant,
    ) -> Result<PlannedDecision, ProxyError> {
        let rate = config.rate(scope);
        let bucket = self.bucket(config, scope, now, rate)?;
        let ordinal = bucket.next_ordinal()?;
        let mut decision = super::plan::decide(config, scope, ordinal);
        let RatePolicy::Limited(rate) = rate else {
            return Ok((ordinal, decision));
        };
        if !bucket.admit(rate, now)? {
            decision.fault = Some(Fault::Throttle);
            self.counters.rate_limited = self.counters.rate_limited.saturating_add(1);
        }
        Ok((ordinal, decision))
    }
    fn bucket(
        &mut self,
        config: &ValidatedConfig,
        scope: &Scope,
        now: Instant,
        rate: RatePolicy,
    ) -> Result<&mut Bucket, ProxyError> {
        if !self.scopes.contains_key(scope) {
            if self.scopes.len() == config.scope_capacity() {
                return Err(ProxyError::Limit("scope capacity"));
            }
            let bucket = Bucket::new(rate, now)?;
            self.scopes.entry(scope.clone()).or_insert(bucket);
        }
        self.scopes
            .get_mut(scope)
            .ok_or(ProxyError::Limit("scope state"))
    }
    fn record(&mut self, config: &ValidatedConfig, entry: DecisionEntry) {
        if let Some(fault) = entry.decision.fault {
            let count = self.counters.faults.entry(fault).or_insert(0);
            *count = count.saturating_add(1);
        }
        if !entry.decision.latency.is_zero() {
            self.counters.delayed = self.counters.delayed.saturating_add(1);
        }
        if self.log.len() == config.log_capacity() && self.log.pop_front().is_some() {
            self.dropped = self.dropped.saturating_add(1);
        }
        self.log.push_back(entry);
    }
}
impl Bucket {
    fn new(rate: RatePolicy, now: Instant) -> Result<Self, ProxyError> {
        let mut bucket = Self {
            ordinal: 0,
            tokens: 0,
            updated: now,
        };
        let RatePolicy::Limited(rate) = rate else {
            return Ok(bucket);
        };
        bucket.tokens = u128::from(rate.get())
            .checked_mul(TOKEN_UNITS)
            .ok_or(ProxyError::Limit("token capacity"))?;
        Ok(bucket)
    }
    fn next_ordinal(&mut self) -> Result<u64, ProxyError> {
        let ordinal = self.ordinal;
        self.ordinal = ordinal
            .checked_add(1)
            .ok_or(ProxyError::Limit("scope ordinal"))?;
        Ok(ordinal)
    }
    fn refill(&mut self, rate: NonZeroU32, now: Instant) -> Result<(), ProxyError> {
        let now = now.max(self.updated);
        let capacity = u128::from(rate.get())
            .checked_mul(TOKEN_UNITS)
            .ok_or(ProxyError::Limit("token capacity"))?;
        let elapsed = now
            .saturating_duration_since(self.updated)
            .as_nanos()
            .min(TOKEN_UNITS);
        let refill = elapsed
            .checked_mul(u128::from(rate.get()))
            .ok_or(ProxyError::Limit("token refill"))?;
        self.tokens = self
            .tokens
            .checked_add(refill)
            .ok_or(ProxyError::Limit("token balance"))?
            .min(capacity);
        self.updated = now;
        Ok(())
    }
    fn admit(&mut self, rate: NonZeroU32, now: Instant) -> Result<bool, ProxyError> {
        self.refill(rate, now)?;
        match self.tokens.checked_sub(TOKEN_UNITS) {
            Some(tokens) => {
                self.tokens = tokens;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
