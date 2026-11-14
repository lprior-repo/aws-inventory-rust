use inventory_core::Resource;
use std::collections::{BTreeMap, BTreeSet};

mod keys;
use keys::{MIN_ARN_COMPONENT_BYTES, arn_component, identity_strings, is_arn};

struct Target {
    id: String,
    account_id: String,
    region: String,
}

pub(super) enum Lookup<'a> {
    Arn(&'a str),

    Identifier(&'a str),

    Ambiguous,

    Unresolved,

    Ignored,
}

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

        if let Some(positions) = self.identifiers.get(value) {
            return self.unique_target(positions, &source.account_id, &source.region, source);
        }
        if self.reference_like(value) {
            Lookup::Unresolved
        } else {
            Lookup::Ignored
        }
    }

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
