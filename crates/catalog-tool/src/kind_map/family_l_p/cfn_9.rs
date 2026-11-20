use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::SMSVOICE::Pool"),
    CfnMapping::canonical("AWS::SMSVOICE::ProtectConfiguration"),
    CfnMapping::canonical("AWS::SMSVOICE::Registration"),
    CfnMapping::canonical("AWS::SMSVOICE::SenderId"),
    CfnMapping::canonical("AWS::SMSVOICE::VerifiedDestinationNumber"),
    CfnMapping::canonical("AWS::SMSVOICE::NotifyConfiguration"),
    CfnMapping::canonical("AWS::SMSVOICE::RcsAgent"),
];
