use inventory_core::catalog::{CatalogError, Service, Shape, Target};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Traversal {
    Scalar,
    List,
}

impl Traversal {
    pub fn merge(self, other: Self) -> Self {
        if self == Self::List || other == Self::List {
            Self::List
        } else {
            Self::Scalar
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ResolvedPointer {
    pub path: String,
    pub traversal: Traversal,
}

#[derive(Debug)]
pub(super) enum Error {
    Segment(String),
    Invalid,
    Terminal,
    Catalog(CatalogError),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Segment(segment) => {
                write!(formatter, "pointer segment {segment} not in item shape")
            }
            Self::Invalid => formatter.write_str("invalid pointer"),
            Self::Terminal => formatter.write_str("pointer does not end in text"),
            Self::Catalog(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<CatalogError> for Error {
    fn from(error: CatalogError) -> Self {
        Self::Catalog(error)
    }
}

pub(super) fn resolve(
    service: Service<'_>,
    item: Target,
    pointer: &str,
) -> Result<ResolvedPointer, Error> {
    let tail = pointer.strip_prefix('/').ok_or(Error::Invalid)?;
    if tail.is_empty() || pointer.len() > 1024 || tail.split('/').count() > 16 {
        return Err(Error::Invalid);
    }
    let (target, path, traversal) = tail.split('/').try_fold(
        (item, String::new(), Traversal::Scalar),
        |(target, mut path, traversal), encoded| {
            let segment = decode(encoded)?;
            if segment == "*" {
                let container = match shape(service, target) {
                    Ok(shape) => shape,
                    Err(Error::Invalid) => return Err(Error::Segment(segment.into_owned())),
                    Err(error) => return Err(error),
                };
                let element = container.element()?;
                return element
                    .map(|target| (target, path, Traversal::List))
                    .ok_or_else(|| Error::Segment(segment.into_owned()));
            }
            let (target, crossed) = member(service, target, &segment)?;
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(&segment);
            Ok((target, path, traversal.merge(crossed)))
        },
    )?;
    let (target, traversal) = match target {
        Target::Shape(index) => {
            let terminal = service.shape(index)?.ok_or(CatalogError::Reference)?;
            if terminal.kind()? == "list" {
                (
                    terminal.element()?.ok_or(CatalogError::Reference)?,
                    Traversal::List,
                )
            } else {
                (target, traversal)
            }
        }
        _ => (target, traversal),
    };
    let terminal = match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(CatalogError::Reference)?
            .kind()?,
        Target::Prelude(kind) => kind,
        _ => return Err(Error::Invalid),
    };
    if path.is_empty() || path.len() > 256 || path.split('.').count() > 8 {
        return Err(Error::Invalid);
    }
    if !matches!(terminal, "string" | "enum") {
        return Err(Error::Terminal);
    }
    Ok(ResolvedPointer { path, traversal })
}

fn decode(segment: &str) -> Result<Cow<'_, str>, Error> {
    if segment.is_empty() {
        return Err(Error::Invalid);
    }
    if !segment.contains('~') {
        return Ok(Cow::Borrowed(segment));
    }
    let mut chars = segment.chars();
    let mut decoded = String::new();
    (0..segment.len()).try_for_each(|_| {
        match chars.next() {
            Some('~') => decoded.push(match chars.next() {
                Some('0') => '~',
                Some('1') => '/',
                _ => return Err(Error::Invalid),
            }),
            Some(character) => decoded.push(character),
            None => {}
        }
        Ok(())
    })?;
    Ok(Cow::Owned(decoded))
}

fn shape(service: Service<'_>, target: Target) -> Result<Shape<'_>, Error> {
    match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(CatalogError::Reference)
            .map_err(Error::from),
        _ => Err(Error::Invalid),
    }
}

fn member(
    service: Service<'_>,
    target: Target,
    segment: &str,
) -> Result<(Target, Traversal), Error> {
    let initial = shape(service, target).map_err(|error| match error {
        Error::Invalid => Error::Segment(segment.to_owned()),
        other => other,
    })?;
    let (container, traversal) = match initial.kind()? {
        "list" => (
            shape(service, initial.element()?.ok_or(CatalogError::Reference)?)?,
            Traversal::List,
        ),
        _ => (initial, Traversal::Scalar),
    };
    container
        .members()?
        .try_fold(None, |found, member| {
            let member = member?;
            if member.name()? == segment {
                return Ok::<_, CatalogError>(Some(member.target));
            }
            Ok(found)
        })?
        .map(|target| (target, traversal))
        .ok_or_else(|| Error::Segment(segment.to_owned()))
}
