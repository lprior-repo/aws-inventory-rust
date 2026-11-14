//! Cold report aggregation; sets/maps contain at most one entry per encoded member/protocol.
use super::{Catalog, CatalogError, Service};
use crate::record::FieldId;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

const FIELDS: [&str; 15] = [
    "services",
    "operations",
    "read_named",
    "readonly",
    "list_describe",
    "list_describe_root",
    "services_with_root",
    "paginated",
    "resources",
    "arn_templates",
    "members",
    "distinct_member_names",
    "json_services",
    "xml_only_services",
    "cbor_services",
];

/// Report counts computed from catalog contents. Allocation occurs only during aggregation.
#[derive(Debug, Default)]
pub struct Counts<'a> {
    values: BTreeMap<&'static str, usize>,
    protocols: BTreeMap<&'a str, usize>,
}
impl<'a> Counts<'a> {
    pub(super) fn read(catalog: Catalog<'a>) -> Result<Self, CatalogError> {
        let mut counts = Self::default();
        let mut names = BTreeSet::new();
        catalog
            .services()?
            .try_for_each(|service| counts.add_service(service?, &mut names))?;
        counts.values.insert("distinct_member_names", names.len());
        Ok(counts)
    }
    /// Read a named report counter; unknown names have no value.
    pub fn get(&self, name: &str) -> Option<usize> {
        if FIELDS.contains(&name) {
            Some(self.count(name))
        } else {
            None
        }
    }
    /// Render the M1.2a report byte-for-byte, including protocol ordering and final newline.
    /// # Errors
    /// Returns a formatting error; data was checked during aggregation.
    pub fn render(&self) -> Result<String, CatalogError> {
        let mut output = String::new();
        FIELDS
            .iter()
            .try_for_each(|name| writeln!(output, "{name} {}", self.count(name)))?;
        self.protocols
            .iter()
            .try_for_each(|(name, count)| writeln!(output, "protocol {name} {count}"))?;
        Ok(output)
    }
    fn count(&self, name: &str) -> usize {
        match self.values.get(name) {
            Some(n) => *n,
            None => 0,
        }
    }
    fn add(&mut self, name: &'static str, n: usize) -> Result<(), CatalogError> {
        let value = self
            .count(name)
            .checked_add(n)
            .ok_or(CatalogError::Overflow)?;
        self.values.insert(name, value);
        Ok(())
    }
    fn add_service(
        &mut self,
        service: Service<'a>,
        names: &mut BTreeSet<FieldId>,
    ) -> Result<(), CatalogError> {
        self.add("services", 1)?;
        self.add_protocols(service)?;
        service.resources()?.try_for_each(|name| {
            name?;
            self.add("resources", 1)
        })?;
        let root = service.operations()?.try_fold(false, |root, operation| {
            let flags = operation?.flags()?;
            self.add_operation(flags)?;
            Ok::<_, CatalogError>(root || flags & 12 == 12)
        })?;
        self.add("services_with_root", usize::from(root))?;
        self.add_shapes(service, names)
    }
    fn add_shapes(
        &mut self,
        service: Service<'a>,
        names: &mut BTreeSet<FieldId>,
    ) -> Result<(), CatalogError> {
        service.shapes()?.try_for_each(|shape| {
            let shape = shape?;
            self.add("arn_templates", usize::from(shape.arn()?.is_some()))?;
            shape.members()?.try_for_each(|member| {
                names.insert(member?.id);
                self.add("members", 1)
            })
        })
    }
    fn add_operation(&mut self, flags: u8) -> Result<(), CatalogError> {
        self.add("operations", 1)?;
        [
            ("readonly", 1),
            ("paginated", 2),
            ("list_describe", 4),
            ("read_named", 16),
        ]
        .into_iter()
        .try_for_each(|(name, mask)| self.add(name, usize::from(flags & mask != 0)))?;
        self.add("list_describe_root", usize::from(flags & 12 == 12))
    }
    fn add_protocols(&mut self, service: Service<'a>) -> Result<(), CatalogError> {
        let (json, cbor, xml) =
            service
                .protocols()?
                .try_fold((false, false, false), |(j, c, x), name| {
                    let name = name?;
                    let previous = match self.protocols.get(name) {
                        Some(n) => *n,
                        None => 0usize,
                    };
                    self.protocols
                        .insert(name, previous.checked_add(1).ok_or(CatalogError::Overflow)?);
                    let (json, cbor, xml) = protocol_kind(name);
                    Ok::<_, CatalogError>((j || json, c || cbor, x || xml))
                })?;
        self.add("json_services", usize::from(json))?;
        self.add("cbor_services", usize::from(cbor))?;
        self.add("xml_only_services", usize::from(xml && !json && !cbor))
    }
}

fn protocol_kind(name: &str) -> (bool, bool, bool) {
    (
        matches!(
            name,
            "aws.protocols#restJson1" | "aws.protocols#awsJson1_0" | "aws.protocols#awsJson1_1"
        ),
        name == "smithy.protocols#rpcv2Cbor",
        matches!(
            name,
            "aws.protocols#awsQuery" | "aws.protocols#ec2Query" | "aws.protocols#restXml"
        ),
    )
}
