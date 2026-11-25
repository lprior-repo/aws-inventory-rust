use super::{NameError, OperationName, OperationOverlay};
use inventory_core::{
    catalog::{
        MAX_CATALOG_BYTES,
        runtime::{
            CatalogSourceProvenance, ModelSourceProvenance, RuntimeCatalog, RuntimeCatalogError,
        },
    },
    record::{FieldId, KeyRef, ListRef, MapRef, Value, ValueRef, encode_container},
};
use sha2::{Digest, Sha256};

const RUNTIME_EXTENSION: u32 = 60_000;
const PLANS: u32 = 2;
const REVIEW_PROVENANCE: u32 = 11;
const SKIPS: u32 = 26;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SkipEvidence {
    pub service: String,
    #[serde(deserialize_with = "operation_name")]
    pub list: OperationName,
    #[serde(deserialize_with = "operation_name")]
    pub describe: OperationName,
    pub inputs_sha256: String,
    pub compared: String,
    pub resources: u32,
    pub when: String,
}

impl SkipEvidence {
    pub fn new(
        service: &str,
        list: &str,
        describe: &str,
        inputs_sha256: &str,
        compared: &str,
        resources: u32,
        when: &str,
    ) -> Result<Self, NameError> {
        if resources == 0 || !text(compared) || !text(when) || !digest_text(inputs_sha256) {
            return Err(NameError {
                value: "invalid skip comparison evidence".to_owned(),
            });
        }
        Ok(Self {
            service: service.to_owned(),
            list: OperationName::new(list)?,
            describe: OperationName::new(describe)?,
            inputs_sha256: inputs_sha256.to_owned(),
            compared: compared.to_owned(),
            resources,
            when: when.to_owned(),
        })
    }

    pub fn fingerprint(
        catalog: &RuntimeCatalog<'_>,
        service: &str,
        list: &str,
        describe: &str,
    ) -> Result<String, RuntimeCatalogError> {
        let root = MapRef::from_container_bytes(catalog.raw_bytes(), MAX_CATALOG_BYTES)?;
        let Some(ValueRef::Map(extension)) = root.get_field(FieldId::new(RUNTIME_EXTENSION)?)?
        else {
            return Err(RuntimeCatalogError::MissingExtension);
        };
        let Some(ValueRef::List(plans)) = extension.get_field(FieldId::new(PLANS)?)? else {
            return Err(RuntimeCatalogError::Capability);
        };
        let mut hash = Sha256::new();
        hash.update(b"catalog-tool-skip-inputs-v1\0");
        hash_pins(&mut hash, catalog.source_provenance());
        [list, describe].into_iter().try_for_each(|operation| {
            let index = catalog
                .plan_index(service, operation)
                .ok_or_else(|| RuntimeCatalogError::Selection(format!("{service}/{operation}")))?;
            hash_plan(&mut hash, plans, index)
        })?;
        Ok(hash
            .finalize()
            .iter()
            .flat_map(|byte| [byte >> 4, byte & 0x0f])
            .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
            .collect())
    }
}

fn operation_name<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<OperationName, D::Error> {
    let name = <String as serde::Deserialize>::deserialize(deserializer)?;
    OperationName::new(&name).map_err(serde::de::Error::custom)
}

fn hash_pins(hash: &mut Sha256, pins: &CatalogSourceProvenance) {
    match &pins.models {
        ModelSourceProvenance::Repository { commit, sha256 } => {
            hash.update([0]);
            hash.update(commit.as_bytes());
            hash.update(sha256);
        }
        ModelSourceProvenance::CompiledCatalog { sha256 } => {
            hash.update([1]);
            hash.update(sha256);
        }
    }
    hash.update(pins.kind_map_sha256);
    match pins.cfn_sha256 {
        Some(sha256) => {
            hash.update([1]);
            hash.update(sha256);
        }
        None => hash.update([0]),
    }
}

fn hash_plan(
    hash: &mut Sha256,
    plans: ListRef<'_>,
    index: usize,
) -> Result<(), RuntimeCatalogError> {
    let Some(ValueRef::Map(plan)) = plans.get(index)? else {
        return Err(RuntimeCatalogError::Capability);
    };
    let fields =
        plan.iter()?
            .try_fold(std::collections::BTreeMap::new(), |mut fields, entry| {
                let (key, value) = entry?;
                if !matches!(
                    key,
                    KeyRef::Field(field) if matches!(field.value(), REVIEW_PROVENANCE | SKIPS)
                ) {
                    fields.insert(key.to_key(), value.to_value()?);
                }
                Ok::<_, RuntimeCatalogError>(fields)
            })?;
    hash.update(encode_container(&Value::Map(fields), MAX_CATALOG_BYTES)?);
    Ok(())
}

fn digest_text(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn validate(plan: &OperationOverlay) -> Result<(), NameError> {
    if !plan.skip.is_empty() && (plan.endpoint_discovery || plan.tag_endpoint.is_some()) {
        return Err(NameError {
            value: "skip: only enumeration plans can carry skips".to_owned(),
        });
    }
    let valid = plan.skip.len() == plan.skip_evidence.len()
        && plan
            .skip_evidence
            .keys()
            .all(|name| plan.skip.contains(name))
        && plan.skip.iter().all(|name| {
            plan.skip_evidence.get(name).is_some_and(|evidence| {
                evidence.resources > 0
                    && text(&evidence.compared)
                    && text(&evidence.when)
                    && digest_text(&evidence.inputs_sha256)
            })
        });
    if !valid {
        return Err(NameError {
            value: "skip_evidence: each skip requires compared, positive resources, and when"
                .to_owned(),
        });
    }
    Ok(())
}

fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt(compared: &str, resources: u32, when: &str) -> Result<SkipEvidence, NameError> {
        Ok(SkipEvidence {
            service: "things".to_owned(),
            list: OperationName::new("ListThings")?,
            describe: OperationName::new("DescribeThing")?,
            inputs_sha256: "a".repeat(64),
            compared: compared.to_owned(),
            resources,
            when: when.to_owned(),
        })
    }

    #[test]
    fn given_endpoint_discovery_carrier_when_generating_policy_then_skip_is_refused()
    -> Result<(), NameError> {
        let child = OperationName::new("DescribeThing")?;
        let evidence = checked_receipt("equal members", "2026-11-24")?;
        let mut plan = OperationOverlay {
            class: Some(crate::overlay::OverlayClass::Enumerate),
            skip: vec![child.clone()],
            skip_evidence: std::collections::BTreeMap::from([(child, evidence)]),
            ..Default::default()
        };
        crate::overlay::ServiceOverlay::new("things", vec![("ListThings", plan.clone())])?;
        plan.endpoint_discovery = true;
        assert!(crate::overlay::ServiceOverlay::new("things", vec![("ListThings", plan)]).is_err());
        Ok(())
    }

    fn checked_receipt(compared: &str, when: &str) -> Result<SkipEvidence, NameError> {
        SkipEvidence::new(
            "things",
            "ListThings",
            "DescribeThing",
            &"a".repeat(64),
            compared,
            1,
            when,
        )
    }

    #[test]
    fn skip_requires_positive_comparison_receipt() -> Result<(), Box<dyn std::error::Error>> {
        let mut plan = OperationOverlay::default();
        let child = OperationName::new("DescribeThing")?;
        plan.skip.push(child.clone());
        assert!(validate(&plan).is_err());
        plan.skip_evidence.insert(
            child.clone(),
            receipt(
                "ListThings.Things[] == DescribeThing.Thing",
                0,
                "2026-11-24T00:00:00Z",
            )?,
        );
        assert!(validate(&plan).is_err());
        if let Some(receipt) = plan.skip_evidence.get_mut(&child) {
            receipt.resources = 2;
        }
        validate(&plan)?;
        Ok(())
    }

    #[test]
    fn given_empty_or_control_receipt_text_when_validated_then_evidence_is_refused()
    -> Result<(), Box<dyn std::error::Error>> {
        for invalid in ["", "   ", "comparison\nresult", "comparison\u{7f}result"] {
            for (compared, when) in [(invalid, "2026-11-24"), ("equal members", invalid)] {
                assert_eq!(
                    checked_receipt(compared, when)
                        .err()
                        .map(|error| error.value),
                    Some("invalid skip comparison evidence".to_owned())
                );
                let child = OperationName::new("DescribeThing")?;
                let mut plan = OperationOverlay::default();
                plan.skip.push(child.clone());
                plan.skip_evidence
                    .insert(child, receipt(compared, 1, when)?);
                assert_eq!(
                    validate(&plan).err().map(|error| error.value),
                    Some(
                        "skip_evidence: each skip requires compared, positive resources, and when"
                            .to_owned()
                    )
                );
            }
        }
        Ok(())
    }

    #[test]
    fn given_receipt_text_byte_boundary_when_constructed_and_validated_then_only_4096_is_accepted()
    -> Result<(), Box<dyn std::error::Error>> {
        let accepted = "x".repeat(4096);
        let refused = "x".repeat(4097);
        let child = OperationName::new("DescribeThing")?;
        for (compared, when, valid) in [
            (accepted.as_str(), "2026-11-24", true),
            ("equal members", accepted.as_str(), true),
            (refused.as_str(), "2026-11-24", false),
            ("equal members", refused.as_str(), false),
        ] {
            let constructed = checked_receipt(compared, when);
            let mut plan = OperationOverlay::default();
            plan.skip.push(child.clone());
            plan.skip_evidence
                .insert(child.clone(), receipt(compared, 1, when)?);
            if valid {
                let receipt = constructed?;
                assert_eq!(receipt.compared, compared);
                assert_eq!(receipt.when, when);
                validate(&plan)?;
            } else {
                assert_eq!(
                    constructed.err().map(|error| error.value),
                    Some("invalid skip comparison evidence".to_owned())
                );
                assert_eq!(
                    validate(&plan).err().map(|error| error.value),
                    Some(
                        "skip_evidence: each skip requires compared, positive resources, and when"
                            .to_owned()
                    )
                );
            }
        }
        Ok(())
    }
}
