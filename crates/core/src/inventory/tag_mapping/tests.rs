use super::{MAX_MAPPING_KEYS, TagMapping, TagMappingError, TagRole};

fn keys(mapping: &TagMapping, role: TagRole) -> Vec<&str> {
    mapping.keys(role).iter().map(String::as_str).collect()
}

#[test]
fn parses_ordered_keys_per_role() -> Result<(), TagMappingError> {
    let mapping = TagMapping::parse(
        "owner = [\"Owner\", \"cost:owner\"]\nteam = [\"Team\"]\napplication = [\"App\"]\n",
    )?;
    assert_eq!(keys(&mapping, TagRole::Owner), vec!["Owner", "cost:owner"]);
    assert_eq!(keys(&mapping, TagRole::Team), vec!["Team"]);
    assert_eq!(keys(&mapping, TagRole::Application), vec!["App"]);
    Ok(())
}

#[test]
fn empty_document_and_empty_arrays_leave_roles_unmapped() -> Result<(), TagMappingError> {
    let empty = TagMapping::parse("")?;
    assert_eq!(empty, TagMapping::default());
    let mapping = TagMapping::parse("owner = []")?;
    assert!(mapping.keys(TagRole::Owner).is_empty());
    Ok(())
}

#[test]
fn role_names_are_persisted_spellings() {
    let names: Vec<&str> = TagRole::ALL.into_iter().map(TagRole::as_str).collect();
    assert_eq!(names, vec!["owner", "team", "application"]);
}

fn rejected(text: &str) -> Option<TagMappingError> {
    TagMapping::parse(text).err()
}

#[test]
fn rejects_unknown_role_names() {
    assert!(matches!(
        rejected("environment = [\"Env\"]"),
        Some(TagMappingError::UnknownRole(name)) if name == "environment"
    ));
}

#[test]
fn rejects_non_array_roles() {
    assert!(matches!(
        rejected("owner = \"Owner\""),
        Some(TagMappingError::NotKeyList("owner"))
    ));
    assert!(matches!(
        rejected("[team]\nkey = \"Team\""),
        Some(TagMappingError::NotKeyList("team"))
    ));
}

#[test]
fn rejects_non_string_items() {
    assert!(matches!(
        rejected("application = [\"App\", 3]"),
        Some(TagMappingError::NotKeyList("application"))
    ));
}

#[test]
fn rejects_invalid_tag_keys() {
    assert!(matches!(
        rejected("owner = [\"\"]"),
        Some(TagMappingError::Key(_))
    ));
    assert!(matches!(
        rejected("owner = [\"bad\\u0007key\"]"),
        Some(TagMappingError::Key(_))
    ));
}

#[test]
fn rejects_duplicate_keys_within_a_role_only() {
    assert!(matches!(
        rejected("team = [\"Team\", \"Team\"]"),
        Some(TagMappingError::DuplicateKey("team"))
    ));
    assert!(TagMapping::parse("owner = [\"Shared\"]\nteam = [\"Shared\"]").is_ok());
}

#[test]
fn rejects_more_than_the_role_bound() {
    let keys: Vec<String> = (0..=MAX_MAPPING_KEYS)
        .map(|index| format!("\"key{index}\""))
        .collect();
    let text = format!("owner = [{}]", keys.join(","));
    assert!(matches!(
        rejected(&text),
        Some(TagMappingError::TooManyKeys("owner"))
    ));
}

#[test]
fn rejects_malformed_toml() {
    assert!(matches!(
        rejected("owner = [\"Owner\""),
        Some(TagMappingError::Syntax)
    ));
}
