use crate::{
    overlay::{MemberPath, Paginator, TokenPair},
    plans::{Model, Pagination, PaginationReason, Vocabulary},
};

pub(super) fn derive(
    model: &Model<'_>,
    operation: &str,
    pagination: &Pagination<'_>,
    vocabulary: &Vocabulary,
) -> Result<Option<Paginator>, String> {
    match pagination {
        Pagination::Modeled { .. } => Ok(None),
        Pagination::Unpaginated => Ok(Some(Paginator::None)),
        Pagination::NeedsOverlay(PaginationReason::TokenBothSides) => {
            pair(model, operation, vocabulary)
        }
        Pagination::NeedsOverlay(reason) => Err(format!("paginator: {reason:?}")),
    }
}

fn pair(
    model: &Model<'_>,
    operation: &str,
    vocabulary: &Vocabulary,
) -> Result<Option<Paginator>, String> {
    let mut pairs = vocabulary.pairs().filter(|(input, output)| {
        present(model, operation, "input", input) && present(model, operation, "output", output)
    });
    let (input, output) = pairs.next().ok_or("paginator: ambiguous token pair")?;
    if pairs.next().is_some() {
        return Err("paginator: ambiguous token pair".to_owned());
    }
    Ok(Some(Paginator::Token(TokenPair::new(
        MemberPath::new(input).map_err(|e| e.to_string())?,
        MemberPath::new(output).map_err(|e| e.to_string())?,
    ))))
}

fn present(model: &Model<'_>, operation: &str, side: &str, path: &str) -> bool {
    model
        .resolve_path(operation, side, path)
        .ok()
        .flatten()
        .is_some()
}
