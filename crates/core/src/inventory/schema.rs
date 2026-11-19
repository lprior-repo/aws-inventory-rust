use super::{InventoryError, Kind, SqlName};
use crate::record::FieldId;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    Integer,
    Real,
    Text,
    Blob,
    Timestamp,
}

impl ScalarType {
    pub fn physical_columns(self) -> usize {
        match self {
            Self::Real | Self::Timestamp => 2,
            _ => 1,
        }
    }
}

#[derive(Debug)]
pub struct Column {
    name: String,
    path: Box<[FieldId]>,
    scalar: ScalarType,
}

impl Column {
    pub fn new(path: Vec<FieldId>, scalar: ScalarType) -> Result<Self, InventoryError> {
        if path.is_empty() || path.len() > 8 {
            return Err(InventoryError::Path);
        }
        let name = path.iter().try_fold(String::from("f"), |mut name, field| {
            write!(&mut name, "_{}", field.value())?;
            Ok::<_, std::fmt::Error>(name)
        })?;
        Ok(Self {
            name,
            path: path.into_boxed_slice(),
            scalar,
        })
    }

    pub fn selected_scalar(scalar: ScalarType) -> Self {
        Self {
            name: "value".to_owned(),
            path: Box::new([]),
            scalar,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn path(&self) -> &[FieldId] {
        &self.path
    }
    pub fn scalar(&self) -> ScalarType {
        self.scalar
    }
}

#[derive(Debug)]
pub struct KindSchema {
    kind: Kind,
    table: SqlName,
    columns: Box<[Column]>,
}

impl KindSchema {
    pub fn new(kind: Kind, table: SqlName, columns: Vec<Column>) -> Result<Self, InventoryError> {
        if table.as_str().starts_with("sqlite_") {
            return Err(InventoryError::ReservedTable);
        }
        Ok(Self {
            kind,
            table,
            columns: canonical_columns(columns)?,
        })
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }
    pub fn table(&self) -> &SqlName {
        &self.table
    }
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }
}

fn canonical_columns(mut columns: Vec<Column>) -> Result<Box<[Column]>, InventoryError> {
    columns.sort_unstable_by(|left, right| left.path.cmp(&right.path));
    if columns
        .windows(2)
        .any(|pair| matches!(pair, [a, b] if a.path == b.path))
    {
        return Err(InventoryError::DuplicateColumn);
    }
    validate_column_budget(&columns)?;
    Ok(columns.into_boxed_slice())
}

/// Maximum columns in a SQLite row, matching SQLITE_MAX_COLUMN build config.
/// The engine adds 7 fixed columns, so this is a hard ceiling on the DDL.
const MAX_SQLITE_COLUMNS: usize = 256;

fn validate_column_budget(columns: &[Column]) -> Result<(), InventoryError> {
    let count = columns.iter().try_fold(7usize, |count, column| {
        count
            .checked_add(column.scalar.physical_columns())
            .ok_or(InventoryError::Columns)
    })?;
    if count > MAX_SQLITE_COLUMNS {
        return Err(InventoryError::Columns);
    }
    Ok(())
}
