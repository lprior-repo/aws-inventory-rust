use super::{Error, OperationFacts};
use serde::{Deserialize, de::IgnoredAny};
use std::{collections::BTreeMap, path::Path};

pub(crate) type Traits = BTreeMap<String, IgnoredAny>;

#[derive(Debug, Deserialize)]
pub(crate) struct Model {
    pub(crate) shapes: BTreeMap<String, Shape>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum Shape {
    Service,
    Operation {
        input: Option<Target>,
        #[serde(default)]
        traits: Traits,
    },
    Structure {
        #[serde(default)]
        members: BTreeMap<String, Member>,
        #[serde(default)]
        mixins: Vec<Target>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Target {
    target: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Member {
    #[serde(default)]
    traits: Traits,
}

impl Model {
    pub fn validate(&self, file: &Path) -> Result<(), Error> {
        match self
            .shapes
            .values()
            .filter(|s| matches!(s, Shape::Service))
            .count()
        {
            0 => Err(crate::smithy::Error::NoService(file.to_path_buf()).into()),
            1 => Ok(()),
            _ => Err(crate::smithy::Error::SeveralServices(file.to_path_buf()).into()),
        }
    }

    pub fn operations<'a>(
        &'a self,
        service: &'a str,
    ) -> impl Iterator<Item = Result<OperationFacts<'a>, Error>> {
        self.shapes.iter().filter_map(move |(id, shape)| {
            let Shape::Operation { input, traits } = shape else {
                return None;
            };
            Some(self.facts(service, id, input, traits))
        })
    }

    pub(crate) fn facts<'a>(
        &'a self,
        service: &'a str,
        id: &'a str,
        input: &Option<Target>,
        traits: &Traits,
    ) -> Result<OperationFacts<'a>, Error> {
        let name = operation_name(id);
        Ok(OperationFacts {
            service,
            name,
            has_required_input: self.required(input, service, name)?,
            readonly: traits.contains_key("smithy.api#readonly"),
        })
    }

    fn required(&self, input: &Option<Target>, service: &str, name: &str) -> Result<bool, Error> {
        let Some(input) = input.as_ref().filter(|i| i.target != "smithy.api#Unit") else {
            return Ok(false);
        };
        let members = self.input_members(&input.target, service, name)?;
        Ok(!crate::smithy::input_has_no_required(
            members
                .values()
                .map(|m| m.traits.contains_key("smithy.api#required")),
        ))
    }

    fn input_members(
        &self,
        target: &str,
        service: &str,
        name: &str,
    ) -> Result<&BTreeMap<String, Member>, Error> {
        let Some(Shape::Structure { members, mixins }) = self.shapes.get(target) else {
            return Err(invalid_input(service, name, target));
        };
        if !mixins.is_empty() {
            return Err(Error::MixinsNotResolved {
                service: service.to_owned(),
                operation: name.to_owned(),
            });
        }
        Ok(members)
    }
}

fn invalid_input(service: &str, name: &str, target: &str) -> Error {
    Error::InvalidInput {
        service: service.to_owned(),
        operation: name.to_owned(),
        target: target.to_owned(),
    }
}

fn operation_name(id: &str) -> &str {
    let Some((_, name)) = id.rsplit_once('#') else {
        return id;
    };
    name
}

pub(crate) fn service<'a>(models: &Path, file: &'a Path) -> Result<&'a str, Error> {
    file.strip_prefix(models)
        .ok()
        .and_then(|p| p.parent())
        .filter(|p| !p.as_os_str().is_empty())
        .and_then(|p| p.components().next())
        .and_then(|c| c.as_os_str().to_str())
        .ok_or_else(|| Error::InvalidModelPath(file.to_path_buf()))
}
