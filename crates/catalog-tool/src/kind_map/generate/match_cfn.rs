use super::Candidate;
use crate::kind_map::{KindMap, KindTarget};
use std::collections::BTreeSet;

pub(super) fn choose(candidate: &Candidate<'_>, cfn: &BTreeSet<String>, hand: &KindMap) -> String {
    if let Some(service) = candidate.cfn_service {
        let kind = format!("AWS::{service}::{}", candidate.resource);
        let explicitly_unmapped = hand
            .lookup(&kind)
            .is_some_and(|entry| matches!(entry.target, KindTarget::Unmapped { .. }));
        if cfn.contains(&kind)
            && !explicitly_unmapped
            && !hand.operations().iter().any(|entry| entry.kind == kind)
        {
            return kind;
        }
    }
    api_only(candidate)
}

fn api_only(candidate: &Candidate<'_>) -> String {
    let mut kind = String::from("AWS::ApiOnly::");
    kind.extend(
        candidate
            .sdk_id
            .chars()
            .filter(|character| *character != ' '),
    );
    kind.push_str(candidate.resource);
    kind
}
