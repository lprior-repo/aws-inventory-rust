use super::SqlError;
use crate::store::{MAX_QUERY_BYTES, MAX_QUERY_CELLS, MAX_QUERY_COLUMNS, MAX_QUERY_ROWS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub rows: usize,
    pub bytes: usize,
    pub cells: usize,
    pub columns: usize,
    pub millis: u64,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            rows: MAX_QUERY_ROWS,
            bytes: MAX_QUERY_BYTES,
            cells: MAX_QUERY_CELLS,
            columns: MAX_QUERY_COLUMNS,
            millis: 5_000,
        }
    }
}

impl Bounds {
    pub fn validate(self) -> Result<(), SqlError> {
        let defaults = Self::default();
        if self.rows == 0
            || self.rows > defaults.rows
            || self.bytes == 0
            || self.bytes > defaults.bytes
            || self.cells == 0
            || self.cells > defaults.cells
            || self.columns == 0
            || self.columns > defaults.columns
            || self.millis == 0
            || self.millis > defaults.millis
        {
            return Err(SqlError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Budget {
    bounds: Bounds,
    rows: usize,
    bytes: usize,
    cells: usize,
}

impl Budget {
    pub fn new(bounds: Bounds) -> Result<Self, SqlError> {
        bounds.validate()?;
        Ok(Self {
            bounds,
            rows: 0,
            bytes: 0,
            cells: 0,
        })
    }

    pub fn charge_columns<'a>(
        &mut self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), SqlError> {
        names.into_iter().enumerate().try_for_each(|(index, name)| {
            charge(index, 1, self.bounds.columns, "columns")?;
            self.charge_bytes(name.len())
        })
    }

    pub fn charge_row(&mut self, columns: usize) -> Result<(), SqlError> {
        let rows = charge(self.rows, 1, self.bounds.rows, "rows")?;
        let cells = charge(self.cells, columns, self.bounds.cells, "cells")?;
        self.rows = rows;
        self.cells = cells;
        Ok(())
    }

    pub fn charge_bytes(&mut self, bytes: usize) -> Result<(), SqlError> {
        self.bytes = charge(self.bytes, bytes, self.bounds.bytes, "bytes")?;
        Ok(())
    }
}

fn charge(
    used: usize,
    amount: usize,
    limit: usize,
    bound: &'static str,
) -> Result<usize, SqlError> {
    used.checked_add(amount)
        .filter(|total| *total <= limit)
        .ok_or(SqlError::ResultTooLarge { bound })
}
