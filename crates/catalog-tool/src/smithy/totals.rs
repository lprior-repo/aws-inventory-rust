use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use super::{
    Error,
    model::ModelFile,
    stats::{ModelStats, compute_stats, service},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Totals {
    pub services: usize,
    pub counts: ModelStats,
    pub services_with_root: usize,
    pub distinct_member_names: HashSet<String>,
    pub json_services: usize,
    pub xml_only_services: usize,
    pub cbor_services: usize,
    pub protocol_counts: BTreeMap<String, usize>,
}

pub(super) fn member_names(model: ModelFile) -> impl Iterator<Item = String> {
    model
        .shapes
        .into_values()
        .flat_map(|shape| shape.into_members().into_keys())
}

impl Totals {
    pub fn add_model(&mut self, model: ModelFile, file: &Path) -> Result<(), Error> {
        self.add_protocols(service(&model, file)?);
        let stats = compute_stats(&model)?;
        self.services = self.services.saturating_add(1);
        self.services_with_root = self
            .services_with_root
            .saturating_add(usize::from(stats.list_describe_root > 0));
        self.add_counts(stats);
        member_names(model).for_each(|name| {
            self.distinct_member_names.insert(name);
        });
        Ok(())
    }

    fn add_counts(&mut self, stats: ModelStats) {
        let counts = &mut self.counts;
        counts.operations = counts.operations.saturating_add(stats.operations);
        counts.read_named = counts.read_named.saturating_add(stats.read_named);
        counts.readonly = counts.readonly.saturating_add(stats.readonly);
        counts.list_describe = counts.list_describe.saturating_add(stats.list_describe);
        counts.list_describe_root = counts
            .list_describe_root
            .saturating_add(stats.list_describe_root);
        counts.paginated = counts.paginated.saturating_add(stats.paginated);
        counts.resources = counts.resources.saturating_add(stats.resources);
        counts.arn_templates = counts.arn_templates.saturating_add(stats.arn_templates);
        counts.members = counts.members.saturating_add(stats.members);
    }

    fn add_protocols(&mut self, service: &super::model::Shape) {
        let has = |ids: &[&str]| ids.iter().any(|id| service.traits.contains_key(*id));
        let json = has(&[
            "aws.protocols#restJson1",
            "aws.protocols#awsJson1_0",
            "aws.protocols#awsJson1_1",
        ]);
        let cbor = has(&["smithy.protocols#rpcv2Cbor"]);
        let xml = has(&[
            "aws.protocols#awsQuery",
            "aws.protocols#ec2Query",
            "aws.protocols#restXml",
        ]);
        self.json_services = self.json_services.saturating_add(usize::from(json));
        self.cbor_services = self.cbor_services.saturating_add(usize::from(cbor));
        self.xml_only_services = self
            .xml_only_services
            .saturating_add(usize::from(!json && !cbor && xml));
        service
            .traits
            .keys()
            .filter(|id| id.contains("protocols#"))
            .for_each(|id| self.count_protocol(id));
    }

    fn count_protocol(&mut self, id: &str) {
        match self.protocol_counts.get_mut(id) {
            Some(count) => *count = count.saturating_add(1),
            None => self.protocol_counts.extend([(id.to_owned(), 1)]),
        }
    }
}
