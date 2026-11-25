use super::super::exclude_members;
use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut service =
        construction::reviewed("ssm-contacts", &[calls_0::CALLS], &[decisions_0::DECISIONS])?;
    exclude_members(
        &mut service,
        "ListContactChannels",
        &["ContactChannels.DeliveryAddress.SimpleAddress"],
    )?;
    Ok(service)
}
