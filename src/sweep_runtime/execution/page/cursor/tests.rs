mod bounds;
mod exact_bounds;
mod malformed;
mod scalars;
mod tuples;
use super::*;
use inventory_core::catalog::runtime::ItemPlan;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn token(input: u32, output: u32) -> Result<TokenBinding, SweepError> {
    let request_path = Box::new([FieldId::new(input)?]);
    let response_path = Box::new([FieldId::new(output)?]);
    Ok(TokenBinding {
        request_placement: TokenPlacement::Body(request_path.clone()),
        response_placement: TokenPlacement::Body(response_path.clone()),
        request_path, response_path, value_type: TokenType::String, optional: false,
    })
}

fn plan() -> Result<PagePlan, SweepError> {
    let mut optional = token(3, 13)?;
    optional.optional = true;
    Ok(PagePlan {
        items: ItemPlan::Root,
        tokens: vec![token(1, 11)?, token(2, 12)?, optional].into_boxed_slice(),
        page_size: None, max_page_size: None,
        truncated_path: Some(Box::new([FieldId::new(14)?])),
    })
}

fn record(entries: impl IntoIterator<Item = (u32, Value)>) -> Result<Record, SweepError> {
    let fields = entries.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<_, inventory_core::record::RecordError>>()?;
    Record::encode(&Value::Map(fields)).map_err(SweepError::from)
}

fn response() -> Result<Record, SweepError> {
    record([(11, Value::Text("name.example.".to_owned())),
        (12, Value::Text("A".to_owned())), (14, Value::Bool(true))])
}

fn advance(plan: &PagePlan, response: &Record) -> Result<Advance, SweepError> {
    Cursor::default().next(plan, response, &http::HeaderMap::new(), 10)?
        .ok_or(SweepError::Operation("expected a continuation"))
}

fn restored(advance: &Advance) -> Result<Cursor, SweepError> {
    Cursor::decode(Some(advance.cursor.as_bytes()))
}
