// Pinned model: .local/aws-models/models/ec2-instance-connect/service/2018-04-02/ec2-instance-connect-2018-04-02.json
use super::support::{reviewed};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ec2-instance-connect",
        vec![

        ],
        &[],
        &[
            // Pushes an SSH public key to the specified EC2 instance for use by the specified user.
            ("SendSSHPublicKey", DenyReason::Mutating),
            // Pushes an SSH public key to the specified EC2 instance.
            ("SendSerialConsoleSSHPublicKey", DenyReason::Mutating),
        ],
    )
}
