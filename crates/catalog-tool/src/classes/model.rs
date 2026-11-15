use super::{Error, OperationFacts};
use serde::{Deserialize, de::IgnoredAny};
use std::{collections::BTreeMap, path::Path};

pub(crate) type Traits = BTreeMap<String, IgnoredAny>;

#[derive(Debug, Deserialize)]
pub(crate) struct Model {
    #[serde(deserialize_with = "crate::model_source::unique_map")]
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
        #[serde(default, deserialize_with = "crate::model_source::unique_map")]
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
            0 => return Err(crate::smithy::Error::NoService(file.to_path_buf()).into()),
            1 => {}
            _ => return Err(crate::smithy::Error::SeveralServices(file.to_path_buf()).into()),
        }
        self.operations("")
            .try_for_each(|facts| facts.map(|_| ()))
            .map_err(|error| match error {
                Error::InputResolution(source) => Error::InvalidInput {
                    file: file.to_path_buf(),
                    source,
                },
                other => other,
            })
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

    fn required(
        &self,
        input: &Option<Target>,
        service: &str,
        operation: &str,
    ) -> Result<bool, Error> {
        crate::model_source::resolve_input(
            self,
            crate::model_source::InputSpec {
                service,
                operation,
                target: input.as_ref().map(|input| input.target.as_str()),
            },
        )
        .map_err(Error::InputResolution)
    }
}

fn operation_name(id: &str) -> &str {
    let Some((_, name)) = id.rsplit_once('#') else {
        return id;
    };
    name
}

pub(crate) fn service<'a>(models: &Path, file: &'a Path) -> Result<&'a str, Error> {
    crate::model_source::service_name(models, file).map_err(Error::from)
}

impl crate::model_source::InputModel for Model {
    fn structure(&self, target: &str) -> Option<bool> {
        self.shapes
            .get(target)
            .map(|shape| matches!(shape, Shape::Structure { .. }))
    }

    fn mixins<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str)) {
        if let Some(Shape::Structure { mixins, .. }) = self.shapes.get(target) {
            mixins.iter().for_each(|mixin| visit(&mixin.target));
        }
    }

    fn members<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str, bool)) {
        if let Some(Shape::Structure { members, .. }) = self.shapes.get(target) {
            members.iter().for_each(|(name, member)| {
                visit(name, member.traits.contains_key("smithy.api#required"));
            });
        }
    }
}
