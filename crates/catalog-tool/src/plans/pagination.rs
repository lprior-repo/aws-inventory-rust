pub use super::types::{LargestPageSize, Pagination, PaginationReason};
use super::{
    Error, Model, PaginatorError,
    model::{target, trait_value},
};
use serde_json::Value;
use std::collections::BTreeSet;

const MAX_VOCABULARY: usize = 4096;
const MAX_PATH_DEPTH: usize = 32;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Vocabulary {
    input: BTreeSet<String>,
    output: BTreeSet<String>,
    pub(super) pairs: BTreeSet<(String, String)>,
}
impl Vocabulary {
    pub(crate) fn pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs
            .iter()
            .map(|(input, output)| (input.as_str(), output.as_str()))
    }

    pub fn record(&mut self, model: &Model<'_>) -> Result<(), Error> {
        model.of_type("operation").try_for_each(|(id, _)| {
            if let Some(Pagination::Modeled {
                input_token,
                output_token,
                ..
            }) = modeled(model, id)?
            {
                self.add(input_token, output_token)?;
            }
            Ok(())
        })
    }

    fn add(&mut self, input: &str, output: &str) -> Result<(), Error> {
        let last = output.rsplit('.').next().map_or(output, |s| s);
        let pair = (input.to_owned(), output.to_owned());
        if self.pairs.len() == MAX_VOCABULARY && !self.pairs.contains(&pair) {
            return Err(Error::Limit);
        }
        insert_name(&mut self.input, input)?;
        insert_name(&mut self.output, last)?;
        self.pairs.insert(pair);
        Ok(())
    }
}

fn insert_name(set: &mut BTreeSet<String>, name: &str) -> Result<(), Error> {
    if set.contains(name) {
        return Ok(());
    }
    if set.len() == MAX_VOCABULARY {
        return Err(Error::Limit);
    }
    set.insert(name.to_owned());
    Ok(())
}

pub fn pagination<'a>(
    model: &Model<'a>,
    id: &str,
    vocabulary: &Vocabulary,
) -> Result<Pagination<'a>, Error> {
    match modeled(model, id)? {
        Some(paginator) => Ok(paginator),
        None => suspected(model, id, vocabulary),
    }
}

fn suspected<'a>(
    model: &Model<'a>,
    id: &str,
    vocabulary: &Vocabulary,
) -> Result<Pagination<'a>, Error> {
    let input = model
        .members(id, "input")?
        .is_some_and(|m| m.keys().any(|k| vocabulary.input.contains(k)));
    let output = model.members(id, "output")?;
    let token = output.is_some_and(|m| m.keys().any(|k| vocabulary.output.contains(k)));
    let flag =
        !(input || token) && output.is_some_and(|m| m.iter().any(|(k, v)| truncation(model, k, v)));
    let reason = match (input, token, flag) {
        (true, true, _) => Some(PaginationReason::TokenBothSides),
        (true, false, _) => Some(PaginationReason::TokenInputOnly),
        (false, true, _) => Some(PaginationReason::TokenOutputOnly),
        (false, false, true) => Some(PaginationReason::TruncationFlag),
        (false, false, false) => None,
    };
    Ok(reason.map_or(Pagination::Unpaginated, Pagination::NeedsOverlay))
}

fn field<'a>(model: &Model<'a>, op: &'a Value, key: &str) -> Option<&'a str> {
    op.get(key)
        .or_else(|| model.paginator.and_then(|s| s.get(key)))
        .and_then(Value::as_str)
}

fn modeled<'a>(model: &Model<'a>, id: &str) -> Result<Option<Pagination<'a>>, Error> {
    let Some(op) = trait_value(model.operation(id)?, "smithy.api#paginated") else {
        return Ok(None);
    };
    let paginator = modeled_fields(model, id, op)?;
    validate_modeled(model, id, &paginator)?;
    Ok(Some(paginator))
}

fn modeled_fields<'a>(model: &Model<'a>, id: &str, op: &'a Value) -> Result<Pagination<'a>, Error> {
    let input_token = field(model, op, "inputToken")
        .ok_or_else(|| invalid(model, id, PaginatorError::MissingToken))?;
    let output_token = field(model, op, "outputToken")
        .ok_or_else(|| invalid(model, id, PaginatorError::MissingToken))?;
    Ok(Pagination::Modeled {
        input_token,
        output_token,
        items: field(model, op, "items"),
        page_size: field(model, op, "pageSize"),
    })
}

fn validate_modeled(model: &Model<'_>, id: &str, paginator: &Pagination<'_>) -> Result<(), Error> {
    let Pagination::Modeled {
        input_token,
        output_token,
        items,
        page_size,
    } = paginator
    else {
        return Ok(());
    };
    [Some(*input_token), Some(*output_token), *items, *page_size]
        .into_iter()
        .flatten()
        .try_for_each(|p| validate_path(model, id, p))?;
    validate_tokens(model, id, input_token, output_token)
}

fn validate_tokens(model: &Model<'_>, id: &str, input: &str, output: &str) -> Result<(), Error> {
    if !model
        .members(id, "input")?
        .is_some_and(|m| m.contains_key(input))
    {
        return Err(invalid(model, id, PaginatorError::InputMember));
    }
    let first = output.split('.').next();
    if !model
        .members(id, "output")?
        .is_some_and(|m| first.is_some_and(|p| m.contains_key(p)))
    {
        return Err(invalid(model, id, PaginatorError::OutputMember));
    }
    Ok(())
}

impl Pagination<'_> {
    pub fn largest_page_size(
        &self,
        model: &Model<'_>,
        id: &str,
    ) -> Result<Option<LargestPageSize>, Error> {
        let Self::Modeled {
            page_size: Some(size),
            ..
        } = self
        else {
            return Ok(None);
        };
        let member = model
            .members(id, "input")?
            .and_then(|m| m.get(*size))
            .ok_or_else(|| invalid(model, id, PaginatorError::InputMember))?;
        let value = maximum(member).or_else(|| {
            target(member)
                .and_then(|t| model.shapes.get(t))
                .and_then(maximum)
        });
        Ok(Some(parse_maximum(model, id, value)?))
    }
}

fn parse_maximum(
    model: &Model<'_>,
    id: &str,
    value: Option<&Value>,
) -> Result<LargestPageSize, Error> {
    match value {
        None => Ok(LargestPageSize::Unbounded),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(LargestPageSize::Known)
            .ok_or_else(|| invalid(model, id, PaginatorError::Range)),
    }
}

fn maximum(value: &Value) -> Option<&Value> {
    trait_value(value, "smithy.api#range").and_then(|r| r.get("max"))
}
fn truncation(model: &Model<'_>, name: &str, member: &Value) -> bool {
    let name = name.to_ascii_lowercase();
    let boolean = target(member).is_some_and(|t| {
        t == "smithy.api#Boolean"
            || model
                .shapes
                .get(t)
                .and_then(|s| s.get("type"))
                .and_then(Value::as_str)
                == Some("boolean")
    });
    boolean && (name.contains("truncated") || name.contains("more"))
}
fn validate_path(model: &Model<'_>, id: &str, path: &str) -> Result<(), Error> {
    if path.len() > 1024
        || path.split('.').count() > MAX_PATH_DEPTH
        || path.split('.').any(str::is_empty)
    {
        return Err(invalid(model, id, PaginatorError::Path));
    }
    Ok(())
}
fn invalid(model: &Model<'_>, id: &str, kind: PaginatorError) -> Error {
    Error::InvalidPaginator {
        service: model.service.to_owned(),
        operation: super::model::name(id).to_owned(),
        kind,
    }
}
