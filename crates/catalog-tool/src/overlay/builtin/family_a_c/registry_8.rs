use super::support::{NameError, ServiceOverlay, service};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        service("cloudwatchomni", [
            super::cloudwatchomni_0::part_0()?,
            super::cloudwatchomni_0::part_1()?,
            super::cloudwatchomni_0::part_2()?,
        ].into_iter().flatten().collect())?,
        service("codeartifact", [
            super::codeartifact_0::part_0()?,
            super::codeartifact_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("codebuild", [
            super::codebuild_0::part_0()?,
            super::codebuild_0::part_1()?,
            super::codebuild_0::part_2()?,
        ].into_iter().flatten().collect())?,
        service("codecatalyst", [
            super::codecatalyst_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("codecommit", [
            super::codecommit_0::part_0()?,
            super::codecommit_0::part_1()?,
            super::codecommit_0::part_2()?,
        ].into_iter().flatten().collect())?,
        service("codeconnections", [
            super::codeconnections_0::part_0()?,
            super::codeconnections_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("codedeploy", [
            super::codedeploy_0::part_0()?,
            super::codedeploy_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("codeguru-reviewer", [
            super::codeguru_reviewer_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("codeguru-security", [
            super::codeguru_security_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("codeguruprofiler", [
            super::codeguruprofiler_0::part_0()?,
        ].into_iter().flatten().collect())?,
    ])
}
