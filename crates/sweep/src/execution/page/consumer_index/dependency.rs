use super::*;

pub(super) fn dependency(source: &BindingSource) -> Result<Dependency<'_>, ConsumerIndexError> {
    let leaf = std::iter::successors(Some(source), |source| match source {
        BindingSource::Transformed { source, .. } => Some(source.as_ref()),
        _ => None,
    })
    .take(MAX_SOURCE_DEPTH)
    .find(|source| !matches!(source, BindingSource::Transformed { .. }))
    .ok_or(ConsumerIndexError::SourceDepth)?;
    staged_dependency(leaf)
}

fn staged_dependency(source: &BindingSource) -> Result<Dependency<'_>, ConsumerIndexError> {
    match source {
        BindingSource::ContextFromBranch {
            service,
            operation,
            branch,
            ..
        } => Ok(Dependency::Producer(Producer {
            service,
            operation,
            branch: Some(branch),
        })),
        _ => ordinary_dependency(source),
    }
}

fn ordinary_dependency(source: &BindingSource) -> Result<Dependency<'_>, ConsumerIndexError> {
    match source {
        BindingSource::Operation {
            service, operation, ..
        }
        | BindingSource::OperationArn {
            service, operation, ..
        }
        | BindingSource::OperationUrl {
            service, operation, ..
        }
        | BindingSource::Context {
            service, operation, ..
        } => Ok(Dependency::Producer(Producer {
            service,
            operation,
            branch: None,
        })),
        _ => other_dependency(source),
    }
}

fn other_dependency(source: &BindingSource) -> Result<Dependency<'_>, ConsumerIndexError> {
    match source {
        BindingSource::ParentIdentity { .. } => Ok(Dependency::AnyProducer),
        BindingSource::Alternatives(alternatives) => {
            if alternatives.is_empty() || alternatives.len() > 8 {
                return Err(ConsumerIndexError::Bounds);
            }
            Ok(Dependency::Alternatives(alternatives))
        }
        BindingSource::Transformed { .. } => Err(ConsumerIndexError::SourceDepth),
        _ => Ok(Dependency::Independent),
    }
}
