pub(crate) const fn supported(version: Option<u32>) -> bool {
    matches!(version, Some(180_000..))
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn accepts_only_known_server_versions_at_or_above_the_floor() {
        assert!(!supported(None));
        assert!(!supported(Some(179_999)));
        assert!(supported(Some(180_000)));
        assert!(supported(Some(180_006)));
        assert!(supported(Some(190_000)));
    }
}
