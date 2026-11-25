use super::super::exclude_members;
use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod decisions_0;
mod decisions_1;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut service = construction::reviewed(
        "ses",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS],
    )?;
    exclude_members(
        &mut service,
        "ListCustomVerificationEmailTemplates",
        &["CustomVerificationEmailTemplates.FromEmailAddress"],
    )?;
    Ok(service)
}
