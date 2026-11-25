use super::support::reviewed;
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "forecastquery",
        vec![],
        &[],
        &[
            ("QueryForecast", DenyReason::Paid),
            ("QueryWhatIfForecast", DenyReason::Paid),
        ],
    )
}
