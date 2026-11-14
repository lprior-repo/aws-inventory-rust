//! Allocation-free, bounded container views.
use super::{FieldId, KeyRef, RecordError, ValueKind, ValueRef, iter, readers, skip};
use minicbor::Decoder;
use std::cmp::Ordering;

/// A borrowed array, starting at its complete CBOR header.
#[derive(Clone, Copy, Debug)]
pub struct ListRef<'a> {
    pub(super) bytes: &'a [u8],
}
/// A borrowed map, starting at its complete CBOR header.
#[derive(Clone, Copy, Debug)]
pub struct MapRef<'a> {
    pub(super) bytes: &'a [u8],
}
impl<'a> MapRef<'a> {
    /// Validate framing and supported, strictly ascending reachable map keys without copying.
    /// Tags and indefinite containers remain opaque; readers check selected value forms.
    /// # Errors
    /// Exactly the errors of `Record::from_bytes`.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RecordError> {
        super::validate(bytes)?;
        Ok(Self { bytes })
    }

    /// Find a field, stopping once validation-established sorted keys pass it.
    /// # Errors
    /// Rejects unsupported keys or selected values.
    pub fn get_field(self, id: FieldId) -> Result<Option<ValueRef<'a>>, RecordError> {
        self.get_key(KeyRef::Field(id))
    }
    /// Find a name, stopping once validation-established sorted keys pass it.
    /// # Errors
    /// Rejects unsupported keys or selected values.
    pub fn get_name(self, name: &str) -> Result<Option<ValueRef<'a>>, RecordError> {
        self.get_key(KeyRef::Name(name))
    }
    fn get_key(self, key: KeyRef<'_>) -> Result<Option<ValueRef<'a>>, RecordError> {
        // Validation proves strict key order in every map a reader can expose.
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.map()?, 0)?;
        let Some(result) = (0..count).find_map(|_| {
            let found = match readers::read_key(&mut decoder) {
                Ok(found) => found,
                Err(err) => return Some(Err(err)),
            };
            match found.cmp(&key) {
                Ordering::Greater => Some(Ok(None)),
                Ordering::Equal => Some(readers::read_value(&mut decoder).map(Some)),
                Ordering::Less => skip::value(&mut decoder).err().map(Err),
            }
        }) else {
            return Ok(None);
        };
        result
    }

    /// Look up a field and check its value kind.
    /// # Errors
    /// Returns `KindMismatch` if the field is present with a different kind.
    pub fn get_typed(
        self,
        id: FieldId,
        expected: ValueKind,
    ) -> Result<Option<ValueRef<'a>>, RecordError> {
        let Some(value) = self.get_field(id)? else {
            return Ok(None);
        };
        let found = value.kind();
        if found == expected {
            return Ok(Some(value));
        }
        Err(RecordError::KindMismatch {
            field: id,
            expected,
            found,
        })
    }
    /// Iterate entries once, ending after the first error.
    /// # Errors
    /// Rejects an indefinite map header; items report unsupported forms.
    pub fn iter(
        self,
    ) -> Result<impl Iterator<Item = Result<(KeyRef<'a>, ValueRef<'a>), RecordError>>, RecordError>
    {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.map()?, 0)?;
        Ok(iter::entries(decoder, count, |decoder| {
            Ok((readers::read_key(decoder)?, readers::read_value(decoder)?))
        }))
    }
}
impl<'a> ListRef<'a> {
    /// Find an index without reading beyond this list's declared length.
    /// # Errors
    /// Returns a read error for an unsupported header or selected item.
    pub fn get(self, index: usize) -> Result<Option<ValueRef<'a>>, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        let target = u64::try_from(index).map_err(|_| RecordError::SizeLimit)?;
        if target >= count {
            return Ok(None);
        }
        (0..target).try_for_each(|_| skip::value(&mut decoder))?;
        readers::read_value(&mut decoder).map(Some)
    }
    /// Iterate items once, ending after the first error.
    /// # Errors
    /// Rejects an indefinite array header; items report unsupported forms.
    pub fn iter(
        self,
    ) -> Result<impl Iterator<Item = Result<ValueRef<'a>, RecordError>>, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        Ok(iter::entries(decoder, count, readers::read_value))
    }
}
