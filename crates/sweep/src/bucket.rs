//! The limiter bucket a call is paced in: one per account, region and quota group.

use inventory_core::inventory::Region;

/// Account segment for the two bootstrap calls: the account is not known before the identity call answers.
pub(crate) const BOOTSTRAP_ACCOUNT: &str = "bootstrap";

/// The key `<account>/<region>/<group>`; an account-wide scope reads `global`.
pub(crate) fn key(account: &str, region: &Region, group: &str) -> String {
    [account, region.as_str(), group].join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::inventory::InventoryError;

    const ACCOUNT: &str = "123456789012";

    #[test]
    fn given_a_regional_scope_when_keyed_then_the_region_name_is_the_middle_part()
    -> Result<(), InventoryError> {
        // Given a regional scope.
        let region = Region::parse("us-east-1")?;
        // When the key is built. Then it reads account, region, group.
        assert_eq!(key(ACCOUNT, &region, "ec2"), "123456789012/us-east-1/ec2");
        Ok(())
    }

    #[test]
    fn given_an_account_wide_scope_when_keyed_then_the_middle_part_is_global() {
        // Given an account-wide scope. When the key is built. Then the region reads `global`.
        assert_eq!(
            key(ACCOUNT, &Region::Global, "iam"),
            "123456789012/global/iam"
        );
    }

    #[test]
    fn given_two_regions_when_keyed_then_the_keys_differ() -> Result<(), InventoryError> {
        // Given two regions of one account and group.
        let east = Region::parse("us-east-1")?;
        let west = Region::parse("eu-west-1")?;
        // When both are keyed. Then each has its own bucket.
        assert_ne!(key(ACCOUNT, &east, "ec2"), key(ACCOUNT, &west, "ec2"));
        Ok(())
    }

    #[test]
    fn given_two_accounts_when_keyed_then_the_keys_differ() -> Result<(), InventoryError> {
        // Given one region and group in two accounts.
        let region = Region::parse("us-east-1")?;
        // When both are keyed. Then each has its own bucket.
        assert_ne!(
            key(ACCOUNT, &region, "ec2"),
            key("234567890123", &region, "ec2")
        );
        Ok(())
    }

    #[test]
    fn given_no_known_account_when_keyed_then_the_account_part_is_bootstrap()
    -> Result<(), InventoryError> {
        // Given the home region before the identity call answered.
        let region = Region::parse("us-east-1")?;
        // When the key is built. Then the account part is the literal `bootstrap`.
        assert_eq!(
            key(BOOTSTRAP_ACCOUNT, &region, "sts"),
            "bootstrap/us-east-1/sts"
        );
        Ok(())
    }

    #[test]
    fn given_the_longest_parts_when_keyed_then_the_key_fits_the_limiter_bound()
    -> Result<(), InventoryError> {
        // Given a 12 digit account, a 15 byte region and a 63 byte group.
        let region = Region::parse("ap-southeast-11")?;
        let group = "g".repeat(63);
        // When the key is built.
        let key = key(ACCOUNT, &region, &group);
        // Then it is 92 bytes, under the limiter's 512.
        assert_eq!(key.len(), 92);
        assert!(key.len() <= 512);
        Ok(())
    }
}
