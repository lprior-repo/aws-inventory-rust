//! Borrowed lists, maps and structs: lookup and iteration without allocation.

use std::cmp::Ordering;

use super::cursor::Cursor;
use super::reader::{ValueRef, read_value, skip_value};
use super::value::FieldId;

/// Splits a container body into its entry count and a cursor over its entries.
fn entries(body: &[u8]) -> (u64, Cursor<'_>) {
    let mut cursor = Cursor::new(body);
    let count = cursor.varint().unwrap_or(0);
    (count, cursor)
}

/// A list inside a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListRef<'a>(&'a [u8]);

impl<'a> ListRef<'a> {
    pub(super) const fn new(body: &'a [u8]) -> Self {
        Self(body)
    }

    /// Number of items.
    #[must_use]
    pub fn len(&self) -> usize {
        usize::try_from(entries(self.0).0).unwrap_or(usize::MAX)
    }

    /// Whether the list has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The items in order.
    #[must_use]
    pub fn iter(&self) -> ListIter<'a> {
        let (remaining, cursor) = entries(self.0);
        ListIter { cursor, remaining }
    }
}

/// Iterator over the items of a [`ListRef`].
#[derive(Debug, Clone)]
pub struct ListIter<'a> {
    cursor: Cursor<'a>,
    remaining: u64,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = ValueRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = self.remaining.checked_sub(1)?;
        read_value(&mut self.cursor)
    }
}

/// A string-keyed map inside a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapRef<'a>(&'a [u8]);

impl<'a> MapRef<'a> {
    pub(super) const fn new(body: &'a [u8]) -> Self {
        Self(body)
    }

    /// The value stored under `key`, skipping every other entry undecoded.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<ValueRef<'a>> {
        let (count, mut cursor) = entries(self.0);
        for _ in 0..count {
            match cursor.blob().ok()?.cmp(key.as_bytes()) {
                Ordering::Equal => return read_value(&mut cursor),
                Ordering::Greater => return None,
                Ordering::Less => skip_value(&mut cursor)?,
            }
        }
        None
    }

    /// The entries in key order.
    #[must_use]
    pub fn iter(&self) -> MapIter<'a> {
        let (remaining, cursor) = entries(self.0);
        MapIter { cursor, remaining }
    }
}

/// Iterator over the entries of a [`MapRef`].
#[derive(Debug, Clone)]
pub struct MapIter<'a> {
    cursor: Cursor<'a>,
    remaining: u64,
}

impl<'a> Iterator for MapIter<'a> {
    type Item = (&'a str, ValueRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = self.remaining.checked_sub(1)?;
        let key = self.cursor.text().ok()?;
        Some((key, read_value(&mut self.cursor)?))
    }
}

/// A structure inside a record, keyed by field id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructRef<'a>(&'a [u8]);

impl<'a> StructRef<'a> {
    pub(super) const fn new(body: &'a [u8]) -> Self {
        Self(body)
    }

    /// The value of field `id`, skipping every other field undecoded.
    #[must_use]
    pub fn get(&self, id: FieldId) -> Option<ValueRef<'a>> {
        let (count, mut cursor) = entries(self.0);
        for _ in 0..count {
            match cursor.field_id().ok()?.cmp(&id) {
                Ordering::Equal => return read_value(&mut cursor),
                Ordering::Greater => return None,
                Ordering::Less => skip_value(&mut cursor)?,
            }
        }
        None
    }

    /// The fields in id order.
    #[must_use]
    pub fn iter(&self) -> StructIter<'a> {
        let (remaining, cursor) = entries(self.0);
        StructIter { cursor, remaining }
    }
}

/// Iterator over the fields of a [`StructRef`].
#[derive(Debug, Clone)]
pub struct StructIter<'a> {
    cursor: Cursor<'a>,
    remaining: u64,
}

impl<'a> Iterator for StructIter<'a> {
    type Item = (FieldId, ValueRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = self.remaining.checked_sub(1)?;
        let id = self.cursor.field_id().ok()?;
        Some((id, read_value(&mut self.cursor)?))
    }
}
