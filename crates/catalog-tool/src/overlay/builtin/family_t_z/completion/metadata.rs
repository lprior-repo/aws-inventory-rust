use super::*;
use crate::overlay::FixedValue;

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    match service.service.as_str() {
        "wellarchitected" => lenses(service),
        "wisdom" => wisdom(service),
        "wickr" => bots(service),
        "voice-id" => voice(service),
        _ => Ok(()),
    }
}

fn lenses(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = inventory("LensSummaries", Identity::Arn(path("LensArn")?), token("NextToken", "NextToken")?, Scope::Region)?;
    plan.fixed.insert(path("LensType")?, FixedValue::String("CUSTOM_SELF".to_owned()));
    plan.queryable = paths(&["LensArn", "LensAlias", "LensName", "LensType", "LensVersion", "Owner", "LensStatus"])?;
    insert(service, "ListLenses", plan)?;
    let mut profiles = inventory("items", Identity::Arn(path("arn")?), token("nextToken", "nextToken")?, Scope::Region)?;
    profiles.tags = Some(path("tags")?);
    insert(service, "ListAgentProfiles", profiles)?;
    let mut goals = inventory("items", correlated(&["profileArn"], &["id"])?, token("nextToken", "nextToken")?, Scope::Region)?;
    bind(&mut goals, "profileArn", "ListAgentProfiles", "arn")?;
    insert(service, "ListAgentGoals", goals)
}

fn wisdom(service: &mut ServiceOverlay) -> Result<(), NameError> {
    // List summary shapes do not contain body/contents/download URL fields from getters.
    [
        ("ListContents", "contentSummaries", Identity::Arn(path("contentArn")?)),
        ("ListQuickResponses", "quickResponseSummaries", Identity::Arn(path("quickResponseArn")?)),
        ("ListImportJobs", "importJobSummaries", correlated(&["knowledgeBaseId"], &["importJobId"])?),
    ].into_iter().try_for_each(|(operation, items, identity)| {
        let mut plan = inventory(items, identity, token("nextToken", "nextToken")?, Scope::Region)?;
        bind(&mut plan, "knowledgeBaseId", "ListKnowledgeBases", "knowledgeBaseId")?;
        if operation != "ListImportJobs" { plan.tags = Some(path("tags")?); }
        insert(service, operation, plan)
    })
}

fn bots(service: &mut ServiceOverlay) -> Result<(), NameError> {
    // Bot.hasChallenge is a boolean; unlike ListUsers there is no inviteCode.
    let mut plan = inventory("bots", correlated(&["networkId"], &["botId"])?, token("nextToken", "nextToken")?, Scope::Region)?;
    bind(&mut plan, "networkId", "ListNetworks", "networkId")?;
    plan.queryable = paths(&["botId", "displayName", "username", "uname", "pubkey", "status", "groupId", "hasChallenge", "suspended", "lastLogin"])?;
    insert(service, "ListBots", plan)
}

fn voice(service: &mut ServiceOverlay) -> Result<(), NameError> {
    // Summary-only operations: no voiceprint, audio or enrollment file payloads.
    [
        ("ListWatchlists", "WatchlistSummaries", "WatchlistId"),
        ("ListSpeakers", "SpeakerSummaries", "GeneratedSpeakerId"),
        ("ListFraudsters", "FraudsterSummaries", "GeneratedFraudsterId"),
        ("ListFraudsterRegistrationJobs", "JobSummaries", "JobId"),
        ("ListSpeakerEnrollmentJobs", "JobSummaries", "JobId"),
    ].into_iter().try_for_each(|(operation, items, id)| {
        let plan = child(items, correlated(&["DomainId"], &[id])?, "ListDomains", "DomainId", "DomainId", true)?;
        insert(service, operation, plan)
    })
}
