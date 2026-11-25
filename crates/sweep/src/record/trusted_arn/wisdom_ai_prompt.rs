// Pinned qconnect 2020-10-19 resource AIPrompt: aws.api#arn.template is
// ai-prompt/{assistantId}/{aiPromptId}; its summary's #Arn supplies lowercase UUID grammar.
// This reviewed resource kind is stricter than the shared #Arn shape's 1–3 UUID suffixes.
pub(super) fn valid(resource: &str) -> bool {
    let mut parts = resource.split('/');
    matches!(parts.next(), Some("ai-prompt"))
        && parts.next().is_some_and(uuid)
        && parts.next().is_some_and(uuid)
        && parts.next().is_none()
}

fn uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                matches!(byte, b'0'..=b'9' | b'a'..=b'f')
            }
        })
}

#[cfg(test)]
mod tests {
    use super::valid;
    const ASSISTANT: &str = "11111111-1111-4111-8111-111111111111";
    const PROMPT: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

    #[test]
    fn given_documented_ai_prompt_resource_when_checked_then_exact_two_uuid_layout_is_required() {
        assert!(valid(&format!("ai-prompt/{ASSISTANT}/{PROMPT}")));
        for resource in [
            format!("assistant/{ASSISTANT}/{PROMPT}"),
            format!("ai-prompt/{ASSISTANT}"),
            format!("ai-prompt/{ASSISTANT}/{PROMPT}:1"),
            format!("ai-prompt/{ASSISTANT}/{PROMPT}/{PROMPT}"),
            format!("ai-prompt/{ASSISTANT}/{}", PROMPT.to_ascii_uppercase()),
            format!("ai-prompt/{ASSISTANT}/{}", PROMPT.replace('-', "")),
            format!("ai-prompt//{PROMPT}"),
            format!("ai-prompt/{ASSISTANT}/{PROMPT}/"),
        ] {
            assert!(!valid(&resource), "{resource}");
        }
    }

    #[test]
    fn given_any_uuid_position_with_wrong_character_when_checked_then_each_operand_is_rejected() {
        for position in 0..PROMPT.len() {
            let malformed: String = PROMPT
                .char_indices()
                .map(|(index, character)| {
                    if index == position {
                        if character == '-' { '0' } else { 'g' }
                    } else {
                        character
                    }
                })
                .collect();
            assert!(
                !valid(&format!("ai-prompt/{malformed}/{PROMPT}")),
                "assistant byte {position}"
            );
            assert!(
                !valid(&format!("ai-prompt/{ASSISTANT}/{malformed}")),
                "prompt byte {position}"
            );
        }
    }

    #[test]
    fn given_lowercase_uuid_syntax_when_checked_then_no_unmodeled_uuid_version_or_variant_is_required()
     {
        assert!(valid(
            "ai-prompt/00000000-0000-0000-0000-000000000000/ffffffff-ffff-ffff-ffff-ffffffffffff"
        ));
    }
}
