//! Interval parsing for the serve command.

use anyhow::{Context, Result};
use std::time::Duration;

pub struct ServeConfig;

impl ServeConfig {
    pub fn parse_interval(input: &str) -> Result<Duration> {
        let input = input.trim();
        let (value, unit) = split_interval(input)?;
        let numeric = value
            .parse::<u64>()
            .with_context(|| format!("interval value '{value}' is not a number"))?;
        let multiplier = match unit {
            "s" => 1,
            "m" => 60,
            "h" => 3600,
            "d" => 86400,
            _ => anyhow::bail!("unknown interval unit '{unit}'"),
        };
        let total_seconds = numeric
            .checked_mul(multiplier)
            .ok_or_else(|| anyhow::anyhow!("interval exceeds u64 range"))?;
        if total_seconds == 0 {
            anyhow::bail!("interval must be at least one second");
        }
        Ok(Duration::from_secs(total_seconds))
    }
}

fn split_interval(input: &str) -> Result<(&str, &str)> {
    if input.is_empty() {
        anyhow::bail!("interval is empty");
    }
    let digits_end = input
        .chars()
        .enumerate()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(input.len());
    if digits_end == 0 {
        anyhow::bail!("interval must start with a number");
    }
    let (value, unit) = input.split_at(digits_end);
    if unit.is_empty() {
        anyhow::bail!("interval must have a unit");
    }
    Ok((value, unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_seconds() -> Result<()> {
        let d = ServeConfig::parse_interval("30s")?;
        assert_eq!(d, Duration::from_secs(30));
        Ok(())
    }

    #[test]
    fn parse_interval_minutes() -> Result<()> {
        let d = ServeConfig::parse_interval("5m")?;
        assert_eq!(d, Duration::from_secs(300));
        Ok(())
    }

    #[test]
    fn parse_interval_hours() -> Result<()> {
        let d = ServeConfig::parse_interval("2h")?;
        assert_eq!(d, Duration::from_secs(7200));
        Ok(())
    }

    #[test]
    fn parse_interval_days() -> Result<()> {
        let d = ServeConfig::parse_interval("1d")?;
        assert_eq!(d, Duration::from_secs(86400));
        Ok(())
    }

    #[test]
    fn parse_interval_zero_rejected() {
        let result = ServeConfig::parse_interval("0m");
        assert!(result.is_err(), "expected error for zero interval");
    }

    #[test]
    fn parse_interval_unknown_unit_rejected() {
        let result = ServeConfig::parse_interval("5w");
        assert!(result.is_err(), "expected error for unknown unit 'w'");
    }

    #[test]
    fn parse_interval_no_unit_rejected() {
        let result = ServeConfig::parse_interval("5");
        assert!(result.is_err(), "expected error for missing unit");
    }
}
