use super::validation::{row_error, valid_operation, valid_path, valid_service};
use super::{BootstrapPurpose, KindMapError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BootstrapEntry {
    pub(super) service: String,
    pub(super) operation: String,
    pub(super) purpose: BootstrapPurpose,
}

pub(super) fn parse(cells: &[&str], line: usize) -> Result<BootstrapEntry, KindMapError> {
    let service = cells.get(2).copied().filter(|value| valid_service(value));
    let operation = cells.get(3).copied().filter(|value| valid_operation(value));
    let (Some(service), Some(operation)) = (service, operation) else {
        return Err(row_error(
            line,
            "bootstrap requires a valid service and operation",
        ));
    };
    let purpose = parse_purpose(cells, line)?;
    Ok(BootstrapEntry {
        service: service.to_owned(),
        operation: operation.to_owned(),
        purpose,
    })
}

fn parse_purpose(cells: &[&str], line: usize) -> Result<BootstrapPurpose, KindMapError> {
    match cells.get(1).copied() {
        Some("account-identity") => account_identity(cells, line),
        Some("region-discovery") => region_discovery(cells, line),
        _ => Err(row_error(line, "unknown bootstrap role")),
    }
}

fn account_identity(cells: &[&str], line: usize) -> Result<BootstrapPurpose, KindMapError> {
    let [_, _, _, _, account_path] = cells else {
        return Err(row_error(line, "invalid account identity role"));
    };
    if !valid_path(account_path) {
        return Err(row_error(line, "invalid account identity role"));
    }
    Ok(BootstrapPurpose::AccountIdentity {
        account_path: (*account_path).to_owned(),
    })
}

fn region_discovery(cells: &[&str], line: usize) -> Result<BootstrapPurpose, KindMapError> {
    let [_, _, _, _, regions_path, region_name_path, status_path] = cells else {
        return Err(row_error(line, "invalid region discovery role"));
    };
    if !valid_path(regions_path) || !valid_path(region_name_path) {
        return Err(row_error(line, "invalid region discovery role"));
    }
    let status = (*status_path != "-").then(|| (*status_path).to_owned());
    if status.as_ref().is_some_and(|path| !valid_path(path)) {
        return Err(row_error(line, "invalid opt-in status path"));
    }
    Ok(BootstrapPurpose::RegionDiscovery {
        regions_path: (*regions_path).to_owned(),
        region_name_path: (*region_name_path).to_owned(),
        opt_in_status_path: status,
    })
}
