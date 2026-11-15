use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::AutoScaling::ScalingPolicy"),
    CfnMapping::canonical("AWS::AutoScaling::ScheduledAction"),
    CfnMapping::canonical("AWS::AutoScalingPlans::ScalingPlan"),
    CfnMapping::canonical("AWS::B2BI::Capability"),
    CfnMapping::canonical("AWS::B2BI::Partnership"),
    CfnMapping::canonical("AWS::B2BI::Profile"),
    CfnMapping::canonical("AWS::BCMDataExports::Export"),
    CfnMapping::canonical("AWS::Backup::BackupPlan"),
    CfnMapping::canonical("AWS::Backup::BackupVault"),
    CfnMapping::canonical("AWS::Backup::Framework"),
    CfnMapping::canonical("AWS::Backup::ReportPlan"),
    CfnMapping::canonical("AWS::Backup::RestoreTestingPlan"),
    CfnMapping::canonical("AWS::Backup::TieringConfiguration"),
    CfnMapping::canonical("AWS::BackupGateway::Hypervisor"),
    CfnMapping::canonical("AWS::Batch::ComputeEnvironment"),
    CfnMapping::canonical("AWS::Batch::ConsumableResource"),
    CfnMapping::canonical("AWS::Batch::JobQueue"),
    CfnMapping::canonical("AWS::Batch::SchedulingPolicy"),
    CfnMapping::canonical("AWS::Batch::ServiceEnvironment"),
    CfnMapping::canonical("AWS::Bedrock::Agent"),
];
