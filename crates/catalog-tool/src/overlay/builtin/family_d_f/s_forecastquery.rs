// Pinned model: .local/aws-models/models/forecastquery/service/2018-06-26/forecastquery-2018-06-26.json
use super::support::{reviewed};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "forecastquery",
        vec![

        ],
        &[],
        &[
            // Retrieves a forecast for a single item, filtered by the supplied criteria.
            ("QueryForecast", DenyReason::Paid),
            // Retrieves a what-if forecast.
            ("QueryWhatIfForecast", DenyReason::Paid),
        ],
    )
}
