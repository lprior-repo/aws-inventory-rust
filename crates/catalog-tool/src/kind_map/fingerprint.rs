use super::{BootstrapPurpose, KindMap, KindTarget};
use sha2::{Digest, Sha256};

pub(super) fn fields<'a>(hash: &mut Sha256, values: impl IntoIterator<Item = &'a str>) {
    values.into_iter().for_each(|value| {
        hash.update(value.as_bytes());
        hash.update([0]);
    });
    hash.update([0xff]);
}

pub(super) fn digest(map: &KindMap) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(map.source_version.to_le_bytes());
    fields(&mut hash, [map.reviewer.as_str(), map.reviewed.as_str()]);
    fields(&mut hash, map.default_unmapped_reason.as_deref());
    map.entries.iter().for_each(|entry| {
        match &entry.target {
            KindTarget::Mapped { kind, table_name } => fields(
                &mut hash,
                ["cfn", &entry.cfn_type, "mapped", kind, table_name.as_str()],
            ),
            KindTarget::Unmapped { reason } => {
                fields(&mut hash, ["cfn", &entry.cfn_type, "unmapped", reason])
            }
        }
        fields(&mut hash, entry.display_name_path.as_deref());
    });
    map.api_only.iter().for_each(|entry| {
        fields(
            &mut hash,
            ["api", &entry.kind, entry.table_name.as_str(), &entry.reason],
        )
    });
    map.operations.iter().for_each(|entry| {
        fields(
            &mut hash,
            ["operation", &entry.service, &entry.operation, &entry.kind],
        )
    });
    map.additional_emissions.iter().for_each(|entry| {
        fields(
            &mut hash,
            [
                "additional_emission",
                &entry.service,
                &entry.operation,
                &entry.emitter_key,
                &entry.kind,
            ],
        );
    });
    super::relation_fingerprint::update(
        &mut hash,
        &map.relations,
        &map.parent_relations,
        &map.compound_relations,
    );
    map.bootstraps.iter().for_each(|entry| {
        fields(&mut hash, ["bootstrap", &entry.service, &entry.operation]);
        match &entry.purpose {
            BootstrapPurpose::AccountIdentity { account_path } => {
                fields(&mut hash, ["account", account_path])
            }
            BootstrapPurpose::RegionDiscovery {
                regions_path,
                region_name_path,
                opt_in_status_path,
            } => {
                fields(&mut hash, ["regions", regions_path, region_name_path]);
                fields(&mut hash, opt_in_status_path.as_deref());
            }
            BootstrapPurpose::Enumeration => fields(&mut hash, ["enumeration"]),
        }
    });
    hash.finalize().into()
}
