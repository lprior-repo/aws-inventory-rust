use super::{
    ApiOnlyKind, BootstrapPurpose, KindEntry, KindMap, KindTarget, RuntimeOperation,
    bootstrap::BootstrapEntry, emissions::ReviewedEmission,
};
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
    hash_header(&mut hash, map);
    hash_entries(&mut hash, &map.entries);
    hash_api_only(&mut hash, &map.api_only);
    hash_operations(&mut hash, &map.operations);
    hash_emissions(&mut hash, &map.additional_emissions);
    super::relation_fingerprint::update(
        &mut hash,
        &map.relations,
        &map.parent_relations,
        &map.compound_relations,
    );
    hash_bootstraps(&mut hash, &map.bootstraps);
    hash.finalize().into()
}

fn hash_header(hash: &mut Sha256, map: &KindMap) {
    hash.update(map.source_version.to_le_bytes());
    fields(hash, [map.reviewer.as_str(), map.reviewed.as_str()]);
    fields(hash, map.default_unmapped_reason.as_deref());
}

fn hash_entries(hash: &mut Sha256, entries: &[KindEntry]) {
    entries.iter().for_each(|entry| {
        match &entry.target {
            KindTarget::Mapped { kind, table_name } => fields(
                hash,
                ["cfn", &entry.cfn_type, "mapped", kind, table_name.as_str()],
            ),
            KindTarget::Unmapped { reason } => {
                fields(hash, ["cfn", &entry.cfn_type, "unmapped", reason])
            }
        }
        fields(hash, entry.display_name_path.as_deref());
    });
}

fn hash_api_only(hash: &mut Sha256, entries: &[ApiOnlyKind]) {
    entries.iter().for_each(|entry| {
        fields(
            hash,
            ["api", &entry.kind, entry.table_name.as_str(), &entry.reason],
        );
        if let Some(path) = &entry.display_name_path {
            fields(hash, ["api-display", &entry.kind, path]);
        }
    });
}

fn hash_operations(hash: &mut Sha256, operations: &[RuntimeOperation]) {
    operations.iter().for_each(|entry| {
        fields(
            hash,
            ["operation", &entry.service, &entry.operation, &entry.kind],
        )
    });
}

fn hash_emissions(hash: &mut Sha256, emissions: &[ReviewedEmission]) {
    emissions.iter().for_each(|entry| {
        fields(
            hash,
            [
                "additional_emission",
                &entry.service,
                &entry.operation,
                &entry.emitter_key,
                &entry.kind,
            ],
        );
    });
}

fn hash_bootstraps(hash: &mut Sha256, bootstraps: &[BootstrapEntry]) {
    bootstraps.iter().for_each(|entry| {
        fields(hash, ["bootstrap", &entry.service, &entry.operation]);
        match &entry.purpose {
            BootstrapPurpose::AccountIdentity { account_path } => {
                fields(hash, ["account", account_path.as_str()])
            }
            BootstrapPurpose::RegionDiscovery {
                regions_path,
                region_name_path,
                opt_in_status_path,
            } => {
                fields(
                    hash,
                    ["regions", regions_path.as_str(), region_name_path.as_str()],
                );
                fields(hash, opt_in_status_path.as_deref());
            }
            BootstrapPurpose::Enumeration => fields(hash, ["enumeration"]),
        }
    });
}
