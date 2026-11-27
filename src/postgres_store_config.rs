use anyhow::{Context, Result};
use inventory_engine::inventory::PgWriteConfig;
use std::env::VarError;
use std::num::NonZeroUsize;

const MIN_ROWS: &str = "INVENTORY_POSTGRES_COPY_MIN_ROWS";
const DIRECT: &str = "INVENTORY_POSTGRES_COPY_DIRECT";

pub fn write_config_from_env() -> Result<PgWriteConfig> {
    write_config(env_value(MIN_ROWS)?.as_deref(), env_value(DIRECT)?.as_deref())
}

fn write_config(min_rows: Option<&str>, direct: Option<&str>) -> Result<PgWriteConfig> {
    let config = match min_rows {
        Some(value) => {
            let threshold = value
                .parse::<NonZeroUsize>()
                .with_context(|| format!("{MIN_ROWS} must be a positive integer"))?;
            PgWriteConfig::default().with_copy_min_rows(threshold)
        }
        None => PgWriteConfig::default(),
    };
    match direct {
        None | Some("0") => Ok(config),
        Some("1") => config.with_direct_copy().context(DIRECT),
        Some(_) => anyhow::bail!("{DIRECT} must be 1 or 0"),
    }
}

fn env_value(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => anyhow::bail!("{name} must be valid UTF-8"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_copy_min_rows_unset_when_configured_then_config_is_default() -> Result<()> {
        assert_eq!(write_config(None, None)?, PgWriteConfig::default());
        Ok(())
    }

    #[test]
    fn given_copy_min_rows_one_when_configured_then_copy_enabled_at_one_row() -> Result<()> {
        assert_eq!(
            write_config(Some("1"), None)?,
            PgWriteConfig::default().with_copy_min_rows(NonZeroUsize::MIN)
        );
        Ok(())
    }

    #[test]
    fn given_copy_min_rows_sixty_four_when_configured_then_threshold_is_set() -> Result<()> {
        let threshold = NonZeroUsize::new(64).context("64 is non-zero")?;
        assert_eq!(
            write_config(Some("64"), None)?,
            PgWriteConfig::default().with_copy_min_rows(threshold)
        );
        Ok(())
    }

    #[test]
    fn given_direct_copy_zero_without_threshold_when_configured_then_config_is_default() -> Result<()> {
        assert_eq!(write_config(None, Some("0"))?, PgWriteConfig::default());
        Ok(())
    }

    #[test]
    fn given_direct_copy_without_threshold_when_configured_then_error_names_setting() {
        let error = write_config(None, Some("1")).err();
        assert!(error.is_some_and(|e| format!("{e:#}").contains(DIRECT)));
    }

    #[test]
    fn given_invalid_direct_copy_value_when_configured_then_error_names_setting() {
        let error = write_config(Some("1"), Some("true")).err();
        assert!(error.is_some_and(|e| format!("{e:#}").contains(DIRECT)));
    }

    #[test]
    fn given_direct_copy_one_with_threshold_when_configured_then_direct_copy_is_enabled() -> Result<()> {
        assert_eq!(
            write_config(Some("1"), Some("1"))?,
            PgWriteConfig::default()
                .with_copy_min_rows(NonZeroUsize::MIN)
                .with_direct_copy()?
        );
        Ok(())
    }

    #[test]
    fn given_direct_copy_zero_with_threshold_when_configured_then_staging_is_used() -> Result<()> {
        assert_eq!(
            write_config(Some("1"), Some("0"))?,
            PgWriteConfig::default().with_copy_min_rows(NonZeroUsize::MIN)
        );
        Ok(())
    }

    #[test]
    fn given_copy_min_rows_zero_when_configured_then_error_names_setting() {
        let error = write_config(Some("0"), None).err();
        assert!(error.is_some_and(|e| format!("{e:#}").contains(MIN_ROWS)));
    }

    #[test]
    fn given_copy_min_rows_non_numeric_when_configured_then_error_names_setting() {
        let error = write_config(Some("abc"), None).err();
        assert!(error.is_some_and(|e| format!("{e:#}").contains(MIN_ROWS)));
    }
}
