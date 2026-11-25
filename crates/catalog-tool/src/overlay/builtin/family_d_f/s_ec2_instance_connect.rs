use super::support::reviewed;
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ec2-instance-connect",
        vec![],
        &[],
        &[
            ("SendSSHPublicKey", DenyReason::Mutating),
            ("SendSerialConsoleSSHPublicKey", DenyReason::Mutating),
        ],
    )
}
