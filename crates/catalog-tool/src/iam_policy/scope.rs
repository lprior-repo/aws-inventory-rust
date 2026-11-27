use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Partition {
    Aws,
    AwsCn,
    AwsUsGov,
    AwsIso,
    AwsIsoB,
    AwsIsoE,
    AwsIsoF,
}

impl Partition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::AwsCn => "aws-cn",
            Self::AwsUsGov => "aws-us-gov",
            Self::AwsIso => "aws-iso",
            Self::AwsIsoB => "aws-iso-b",
            Self::AwsIsoE => "aws-iso-e",
            Self::AwsIsoF => "aws-iso-f",
        }
    }
}

impl FromStr for Partition {
    type Err = ScopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "aws" => Ok(Self::Aws),
            "aws-cn" => Ok(Self::AwsCn),
            "aws-us-gov" => Ok(Self::AwsUsGov),
            "aws-iso" => Ok(Self::AwsIso),
            "aws-iso-b" => Ok(Self::AwsIsoB),
            "aws-iso-e" => Ok(Self::AwsIsoE),
            "aws-iso-f" => Ok(Self::AwsIsoF),
            _ => Err(ScopeError::Partition),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountId(String);

impl AccountId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AccountId {
    type Err = ScopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() == 12 && value.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(Self(value.to_owned()))
        } else {
            Err(ScopeError::Account)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ScopeError {
    #[error("partition must be a supported AWS partition")]
    Partition,
    #[error("account must contain exactly 12 ASCII digits")]
    Account,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_supported_partition_when_parsed_then_typed_partition_is_returned() {
        assert_eq!("aws-us-gov".parse::<Partition>(), Ok(Partition::AwsUsGov));
        assert!("aws-private".parse::<Partition>().is_err());
    }

    #[test]
    fn given_account_when_parsed_then_only_twelve_digits_are_accepted() {
        assert_eq!(
            "123456789012".parse::<AccountId>().map(|id| id.0),
            Ok("123456789012".to_owned())
        );
        assert!("12345678901".parse::<AccountId>().is_err());
        assert!("12345678901x".parse::<AccountId>().is_err());
    }
}
