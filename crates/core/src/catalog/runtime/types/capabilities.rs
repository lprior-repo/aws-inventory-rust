use crate::{inventory::{AccountId, Kind, Region, RegionId}, record::{FieldId, MapRef, Record, ValueRef}};
use super::super::RuntimeCatalogError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceProjectionPlan {
    IamTrustSummary { path: Box<[FieldId]> },
    SensitiveDigest { path: Box<[FieldId]> },
    SensitiveMapDigest { path: Box<[FieldId]>, key: String },
}

impl ResourceProjectionPlan {
    pub fn protected_path(&self) -> &[FieldId] {
        match self {
            Self::IamTrustSummary { path } | Self::SensitiveDigest { path }
                | Self::SensitiveMapDigest { path, .. } => path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindClassificationPlan {
    pub path: Box<[FieldId]>,
    pub variants: Box<[(String, Kind)]>,
}

impl KindClassificationPlan {
    pub fn classify(&self, record: &Record) -> Result<&Kind, RuntimeCatalogError> {
        let Some(ValueRef::Text(value)) = path_value(record.root(), &self.path)? else {
            return Err(RuntimeCatalogError::Capability);
        };
        self.variants.iter().find(|(variant, _)| variant == value)
            .map(|(_, kind)| kind).ok_or_else(|| RuntimeCatalogError::UnclassifiedKind {
                discriminator: value.to_owned(),
            })
    }
}

fn path_value<'a>(root: MapRef<'a>, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, RuntimeCatalogError> {
    let (last, parents) = path.split_last().ok_or(RuntimeCatalogError::Capability)?;
    let parent = parents.iter().try_fold(Some(root), |current, field| {
        current.map_or(Ok(None), |current| match current.get_field(*field)? {
            Some(ValueRef::Map(map)) => Ok(Some(map)),
            None | Some(ValueRef::Null) => Ok(None),
            _ => Err(RuntimeCatalogError::Capability),
        })
    })?;
    parent.map_or(Ok(None), |parent| parent.get_field(*last).map_err(RuntimeCatalogError::Record))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArnRegionRule { Source, Global }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArnOwnerRule {
    pub partition: String,
    pub service: String,
    pub resource_prefix: String,
    pub region: ArnRegionRule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceOwnerRule { ExecutionAccount, Arn(ArnOwnerRule) }

impl ArnOwnerRule {
    /// Parse the complete reviewed ARN grammar; never infer an owner from arbitrary text.
    pub fn owner(&self, arn: &str, source: &Region) -> Result<(AccountId, Region), RuntimeCatalogError> {
        if arn.len() > 4096 || !arn.is_ascii() {
            return Err(RuntimeCatalogError::Capability);
        }
        let mut parts = arn.splitn(6, ':');
        if parts.next() != Some("arn") || parts.next() != Some(self.partition.as_str())
            || parts.next() != Some(self.service.as_str()) {
            return Err(RuntimeCatalogError::Capability);
        }
        let region = parts.next().ok_or(RuntimeCatalogError::Capability)?;
        let account = AccountId::parse(parts.next().ok_or(RuntimeCatalogError::Capability)?)?;
        let resource = parts.next().ok_or(RuntimeCatalogError::Capability)?;
        let suffix = resource.strip_prefix(&self.resource_prefix).ok_or(RuntimeCatalogError::Capability)?;
        if suffix.is_empty() || suffix.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || matches!(byte, b':' | b'/' | b'\\' | b'*' | b'?')) {
            return Err(RuntimeCatalogError::Capability);
        }
        let region = match (self.region, source) {
            (ArnRegionRule::Global, _) if region.is_empty() => Region::Global,
            (ArnRegionRule::Source, Region::Regional(expected)) if region == expected.as_str() => {
                Region::Regional(RegionId::parse(region)?)
            }
            _ => return Err(RuntimeCatalogError::Capability),
        };
        Ok((account, region))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreign_sink_owner_is_not_execution_account() -> Result<(), Box<dyn std::error::Error>> {
        let rule = ArnOwnerRule { partition: "aws".into(), service: "oam".into(), resource_prefix: "sink/".into(), region: ArnRegionRule::Source };
        let source = Region::Regional(RegionId::parse("us-east-1")?);
        assert_eq!(rule.owner("arn:aws:oam:us-east-1:222222222222:sink/abc", &source)?.0.as_str(), "222222222222");
        ["arn:aws-cn:oam:us-east-1:222222222222:sink/abc", "arn:aws:iam:us-east-1:222222222222:sink/abc", "arn:aws:oam:us-west-2:222222222222:sink/abc", "arn:aws:oam:us-east-1:222222222222:link/abc", "arn:aws:oam:us-east-1:222222222222:sink/", "arn:aws:oam:us-east-1:222222222222:sink/abc:extra", "arn:aws:oam:us-east-1:222222222222:sink/*", "arn:aws:oam:us-east-1:222222222222:sink/targe?"]
            .into_iter().for_each(|arn| assert!(rule.owner(arn, &source).is_err(), "{arn}"));
        Ok(())
    }
    #[test]
    fn discriminator_rejects_unknown_and_missing_variants() -> Result<(), Box<dyn std::error::Error>> {
        use crate::record::{Key, Value};
        let field = FieldId::new(1)?;
        let plan = KindClassificationPlan { path: vec![field].into_boxed_slice(), variants: vec![("PROVISIONED".into(), Kind::parse("AWS::MSK::Cluster")?), ("SERVERLESS".into(), Kind::parse("AWS::MSK::ServerlessCluster")?)].into_boxed_slice() };
        let row = |value: &str| Record::encode(&Value::Map([(Key::Field(field), Value::Text(value.into()))].into_iter().collect()));
        assert_eq!(plan.classify(&row("SERVERLESS")?)?.as_str(), "AWS::MSK::ServerlessCluster");
        assert!(plan.classify(&row("UNKNOWN")?).is_err());
        assert!(plan.classify(&Record::encode(&Value::Map(Default::default()))?).is_err());
        Ok(())
    }
}
