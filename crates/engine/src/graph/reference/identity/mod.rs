//! Scan-local identity index built from observed resources.
use inventory_core::Resource;
use std::collections::{BTreeMap, BTreeSet};

mod keys;
use keys::{MIN_ARN_COMPONENT_BYTES, arn_component, identity_strings, is_arn};

/// One indexed observed resource.
struct Target {
    id: String,
    account_id: String,
    region: String,
}

/// Outcome of resolving one candidate string.
pub(super) enum Lookup<'a> {
    /// Exact ARN match against an observed resource identity.
    Arn(&'a str),
    /// Unique identifier match in the same account and region.
    ///
    /// This is the `reference=identifier` edge: an exact match against a
    /// provider primary identifier of at least `MIN_IDENTIFIER_BYTES`, a full
    /// observed resource identity, or an ARN trailing component. It is a
    /// unique-in-scope match, never a guessed one; below-floor provider
    /// identifiers are not indexed at all.
    Identifier(&'a str),
    /// The identifier names more than one observed resource.
    Ambiguous,
    /// The value looks like a reference but no observed resource owns it.
    Unresolved,
    /// Not a reference, a self-reference, or a duplicate of the source edge.
    Ignored,
}

/// Identities of every resource observed in one scan.
#[derive(Default)]
pub(super) struct IdentityIndex {
    arns: BTreeMap<String, usize>,
    identifiers: BTreeMap<String, Vec<usize>>,
    reference_prefixes: BTreeSet<String>,
    targets: Vec<Target>,
    seen: BTreeMap<String, usize>,
}

impl IdentityIndex {
    pub(super) fn build(resources: &[Resource]) -> Self {
        let mut index = Self::default();
        for resource in resources {
            index.insert(resource);
        }
        index
    }

    /// Add one observed resource. Streaming callers build the index in one
    /// pass without materializing the scan's resources.
    pub(super) fn insert(&mut self, resource: &Resource) {
        if self.seen.contains_key(resource.id.as_str()) {
            return;
        }
        let position = self.targets.len();
        self.seen.insert(resource.id.clone(), position);
        self.targets.push(Target {
            id: resource.id.clone(),
            account_id: resource.account_id.clone(),
            region: resource.region.clone(),
        });
        if is_arn(&resource.id) {
            self.arns.insert(resource.id.clone(), position);
        }
        for identity in identity_strings(resource) {
            self.index_identity(identity, position);
        }
    }

    /// Record one identity string for the target at `position`, remembering
    /// the hyphen-delimited prefix it answers to.
    fn index_identity(&mut self, identity: &str, position: usize) {
        self.identifiers
            .entry(identity.to_owned())
            .or_default()
            .push(position);
        if let Some((prefix, _)) = identity.rsplit_once('-')
            && !prefix.is_empty()
        {
            self.reference_prefixes.insert(prefix.to_owned());
        }
    }

    /// Resolve one candidate against the index.
    ///
    /// ARNs try the exact identity first. A miss falls back to the ARN's final
    /// resource component (the Cloud Control primary identifier for types such
    /// as SNS topics), but only against resources in the ARN's own account and
    /// region. A shared component is ambiguous, never a guessed edge.
    pub(super) fn lookup<'a>(
        &'a self,
        source: &Resource,
        value: &str,
        max_reference_bytes: usize,
    ) -> Lookup<'a> {
        if value.len() > max_reference_bytes {
            return Lookup::Ignored;
        }
        if is_arn(value) {
            return self.lookup_arn(source, value);
        }
        // Provider primary identifiers below `MIN_IDENTIFIER_BYTES` were never
        // inserted, so a short plain value (an enum such as `abc`) can only
        // reach this map through a full observed identity or an ARN component.
        if let Some(positions) = self.identifiers.get(value) {
            return self.unique_target(positions, &source.account_id, &source.region, source);
        }
        if self.reference_like(value) {
            Lookup::Unresolved
        } else {
            Lookup::Ignored
        }
    }

    /// Exact ARN identity, then the ARN's trailing component in its own scope.
    ///
    /// A synthesized-id resource can own an ARN-shaped primary identifier.
    /// Match the full ARN first, scoped to the ARN's own account and region,
    /// then fall through to its component.
    fn lookup_arn<'a>(&'a self, source: &Resource, value: &str) -> Lookup<'a> {
        if let Some(&position) = self.arns.get(value) {
            return self.exact_target(position, source);
        }
        let Some((account_id, region, component)) = arn_component(value) else {
            return Lookup::Unresolved;
        };
        if let Some(positions) = self.identifiers.get(value) {
            match self.unique_target(positions, account_id, region, source) {
                Lookup::Unresolved => {}
                resolved => return resolved,
            }
        }
        if component.len() < MIN_ARN_COMPONENT_BYTES {
            return Lookup::Unresolved;
        }
        match self.identifiers.get(component) {
            Some(positions) => self.unique_target(positions, account_id, region, source),
            None => Lookup::Unresolved,
        }
    }

    fn exact_target<'a>(&'a self, position: usize, source: &Resource) -> Lookup<'a> {
        match self.targets.get(position) {
            Some(target) if target.id == source.id => Lookup::Ignored,
            Some(target) => Lookup::Arn(&target.id),
            None => Lookup::Unresolved,
        }
    }

    /// Ambiguity is judged across the whole scan, including the source itself.
    /// An identity shared by two resources never produces a guessed edge.
    fn unique_target<'a>(
        &'a self,
        positions: &[usize],
        account_id: &str,
        region: &str,
        source: &Resource,
    ) -> Lookup<'a> {
        let (count, only) = self.scoped_targets(positions, account_id, region);
        match (count, only) {
            (0, _) => Lookup::Unresolved,
            (1, Some(target)) if target.id == source.id => Lookup::Ignored,
            (1, Some(target)) => Lookup::Identifier(&target.id),
            _ => Lookup::Ambiguous,
        }
    }

    /// Count the indexed targets inside one account and region, keeping the
    /// first one seen.
    fn scoped_targets<'a>(
        &'a self,
        positions: &[usize],
        account_id: &str,
        region: &str,
    ) -> (usize, Option<&'a Target>) {
        let mut count: usize = 0;
        let mut only: Option<&Target> = None;
        for position in positions {
            let Some(target) = self.targets.get(*position) else {
                continue;
            };
            if target.account_id != account_id || target.region != region {
                continue;
            }
            count = count.saturating_add(1);
            if only.is_none() {
                only = Some(target);
            }
        }
        (count, only)
    }

    /// Prefix lookup is linear in the number of hyphens in the candidate, not
    /// in the number of prefixes known to the scan.
    fn reference_like(&self, value: &str) -> bool {
        value
            .char_indices()
            .filter(|(_, character)| *character == '-')
            .any(|(index, _)| match value.get(..index) {
                Some(prefix) => self.reference_prefixes.contains(prefix),
                None => false,
            })
    }
}
