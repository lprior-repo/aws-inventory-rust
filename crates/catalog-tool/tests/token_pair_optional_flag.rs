use catalog_tool::overlay::{MemberPath, NameError, TokenPair};

#[test]
fn token_pair_optional_flag_is_exactly_a_bool() -> Result<(), NameError> {
    let pin: fn(&mut TokenPair) -> &mut bool = |pair| &mut pair.optional;
    let mut pair = TokenPair::new(MemberPath::new("Start")?, MemberPath::new("Next")?);
    assert!(!pair.optional);
    *pin(&mut pair) = true;
    assert!(pair.optional);
    Ok(())
}
