use super::InventoryError;
use std::sync::Arc;

macro_rules! identifier {
    ($name:ident, $error:ident, $validate:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Arc<str>);

        impl $name {
            pub fn parse(value: &str) -> Result<Self, InventoryError> {
                if !$validate(value) {
                    return Err(InventoryError::$error);
                }
                Ok(Self(Arc::from(value)))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

identifier!(AccountId, Account, account);
identifier!(RegionId, Region, regional);
identifier!(Kind, Kind, kind);
identifier!(ResourceId, Resource, resource);
identifier!(ServiceId, Service, service);
identifier!(OperationId, Operation, operation);
identifier!(Relation, Relation, relation);
identifier!(SqlName, SqlName, sql_name);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    Global,
    Regional(RegionId),
}

impl Region {
    pub fn parse(value: &str) -> Result<Self, InventoryError> {
        if value == "global" {
            return Ok(Self::Global);
        }
        RegionId::parse(value).map(Self::Regional)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Global => "global",
            Self::Regional(region) => region.as_str(),
        }
    }
}

impl Ord for Region {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Region {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn account(value: &str) -> bool {
    value.len() == 12 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn regional(value: &str) -> bool {
    let Some((prefix, number)) = value.rsplit_once('-') else {
        return false;
    };
    value.len() <= 64
        && !prefix.is_empty()
        && !number.is_empty()
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        && prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
        && prefix.as_bytes().last().is_some_and(u8::is_ascii_lowercase)
}

fn kind(value: &str) -> bool {
    if value.len() > 256 {
        return false;
    }
    let mut parts = value.split("::");
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(namespace), Some(service), Some(name), None) => [namespace, service, name]
            .into_iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_alphanumeric())),
        _ => false,
    }
}

fn resource(value: &str) -> bool {
    !value.is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control)
}

fn service(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn operation(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn relation(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-:.".contains(&byte))
}

fn sql_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || *byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
