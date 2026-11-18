// Pinned Smithy source: .local/aws-models/models/payment-cryptography-data/service/2022-02-03/payment-cryptography-data-2022-02-03.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &[
        "DecryptData",
        "EncryptData",
        "GenerateAs2805KekValidation",
        "GenerateAuthRequestCryptogram",
        "GenerateCardValidationData",
        "GenerateMac",
        "GenerateMacEmvPinChange",
        "GeneratePinData",
        "ReEncryptData",
        "TranslateKeyMaterial",
        "TranslatePinData",
        "VerifyAuthRequestCryptogram",
        "VerifyCardValidationData",
        "VerifyMac",
        "VerifyPinData",
    ],
    class: OverlayClass::Deny(DenyReason::Paid),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("payment-cryptography-data", GROUPS, &[])
}
