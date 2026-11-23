use super::*;
use inventory_core::{catalog::Shape, record::Key};

enum Task<'a> {
    Visit(Target, &'a SeedInputValue),
    Finish(Option<Vec<Key>>, usize),
}

struct Compiler<'a, 's> {
    catalog: Catalog<'a>,
    scenario: &'s SeedScenario,
    service: Service<'a>,
}

pub(super) fn prepare<'a>(
    catalog: Catalog<'a>,
    scenario: &SeedScenario,
    operation: SeedOperation<'a>,
    field: &FieldPath,
    value: &SeedInputValue,
) -> Result<PreparedInputValue, SeedError> {
    let target = members::path_target(operation.service, operation.operation.input()?, field)?;
    let compiler = Compiler {
        catalog,
        scenario,
        service: operation.service,
    };
    compiler.compile(target, value)
}

impl Compiler<'_, '_> {
    fn compile(
        &self,
        target: Target,
        value: &SeedInputValue,
    ) -> Result<PreparedInputValue, SeedError> {
        let mut tasks = vec![Task::Visit(target, value)];
        let mut values = Vec::new();
        (0..514)
            .find_map(|_| self.advance(&mut tasks, &mut values))
            .ok_or(SeedError::Invalid("seed preparation exceeds node limit"))?
    }

    fn advance<'a>(
        &self,
        tasks: &mut Vec<Task<'a>>,
        values: &mut Vec<PreparedInputValue>,
    ) -> Option<Result<PreparedInputValue, SeedError>> {
        let result = match tasks.pop() {
            Some(Task::Visit(target, value)) => self.visit(target, value, tasks, values),
            Some(Task::Finish(keys, count)) => finish(keys, count, values),
            None => {
                return Some(
                    values
                        .pop()
                        .ok_or(SeedError::Invalid("prepared seed value is missing")),
                );
            }
        };
        result.err().map(Err)
    }

    fn visit<'a>(
        &self,
        target: Target,
        value: &'a SeedInputValue,
        tasks: &mut Vec<Task<'a>>,
        values: &mut Vec<PreparedInputValue>,
    ) -> Result<(), SeedError> {
        match value {
            SeedInputValue::List(items) => {
                let element = shape(self.service, target)?
                    .element()?
                    .ok_or(SeedError::Invalid("seed list targets a non-list shape"))?;
                tasks.push(Task::Finish(None, items.len()));
                tasks.extend(items.iter().rev().map(|value| Task::Visit(element, value)));
            }
            SeedInputValue::Structure(fields) => self.fields(target, fields, tasks)?,
            value => values.push(self.scalar(target, value)?),
        }
        Ok(())
    }

    fn scalar(
        &self,
        target: Target,
        value: &SeedInputValue,
    ) -> Result<PreparedInputValue, SeedError> {
        match value {
            SeedInputValue::Text(value) => Ok(PreparedInputValue::Text(value.clone())),
            SeedInputValue::Boolean(value) => Ok(PreparedInputValue::Boolean(*value)),
            SeedInputValue::Integer(value) => Ok(PreparedInputValue::Integer(*value)),
            SeedInputValue::Literal(value) => Ok(PreparedInputValue::Literal(
                super::super::literal::resolve(self.service, target, value)?,
            )),
            SeedInputValue::Template(value) => {
                template::prepare(self.catalog, self.scenario, self.service, target, value)
            }
            SeedInputValue::PreviousOutput { step, path } => {
                previous_output(self.catalog, self.scenario, *step, path)
            }
            _ => Err(SeedError::Invalid("seed container was not prepared")),
        }
    }

    fn fields<'a>(
        &self,
        target: Target,
        fields: &'a [(String, SeedInputValue)],
        tasks: &mut Vec<Task<'a>>,
    ) -> Result<(), SeedError> {
        let mut keys = Vec::with_capacity(fields.len());
        let children = fields
            .iter()
            .map(|(name, value)| {
                let (key, target) = self.field(target, name)?;
                keys.push(key);
                Ok(Task::Visit(target, value))
            })
            .collect::<Result<Vec<_>, SeedError>>()?;
        tasks.push(Task::Finish(Some(keys), fields.len()));
        tasks.extend(children.into_iter().rev());
        Ok(())
    }

    fn field(&self, target: Target, name: &str) -> Result<(Key, Target), SeedError> {
        let shape = shape(self.service, target)?;
        match shape.kind()? {
            "structure" | "union" => {
                let member = members::member(self.service, target, name)?;
                Ok((Key::Field(member.id), member.target))
            }
            "map" => Ok((
                Key::Name(name.to_owned()),
                shape
                    .map_value()?
                    .ok_or(SeedError::Invalid("seed map value shape is missing"))?,
            )),
            _ => Err(SeedError::Invalid("seed structure targets a non-map shape")),
        }
    }
}

fn shape(service: Service<'_>, target: Target) -> Result<Shape<'_>, SeedError> {
    let Target::Shape(index) = target else {
        return Err(SeedError::Invalid(
            "seed container requires a modeled shape",
        ));
    };
    service
        .shape(index)?
        .ok_or(SeedError::Invalid("seed container shape is missing"))
}

fn finish(
    keys: Option<Vec<Key>>,
    count: usize,
    values: &mut Vec<PreparedInputValue>,
) -> Result<(), SeedError> {
    let start = values
        .len()
        .checked_sub(count)
        .ok_or(SeedError::Invalid("prepared seed container is incomplete"))?;
    let children = values.split_off(start);
    values.push(match keys {
        Some(keys) => PreparedInputValue::Structure(keys.into_iter().zip(children).collect()),
        None => PreparedInputValue::List(children),
    });
    Ok(())
}
