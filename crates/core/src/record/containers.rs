use super::{FieldId, KeyRef, RecordError, ValueKind, ValueRef, iter, readers, skip};
use minicbor::Decoder;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct ListRef<'a> {
    pub(super) bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct MapRef<'a> {
    pub(super) bytes: &'a [u8],
}
impl<'a> MapRef<'a> {
    pub fn as_bytes(self) -> Result<&'a [u8], RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        skip::value(&mut decoder)?;
        self.bytes
            .get(..decoder.position())
            .ok_or(RecordError::Missing {
                offset: decoder.position(),
            })
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RecordError> {
        super::validate(bytes)?;
        Ok(Self { bytes })
    }

    pub fn from_bytes_bounded(bytes: &'a [u8]) -> Result<Self, RecordError> {
        Self::from_container_bytes(bytes, super::MAX_RECORD_BYTES)
    }

    /// Depth-bounded open of a container bounded by `max_bytes`; the decode
    /// counterpart of [`super::encode_container`].
    pub fn from_container_bytes(bytes: &'a [u8], max_bytes: usize) -> Result<Self, RecordError> {
        super::validate_with_depth(bytes, Some(super::MAX_DEPTH), max_bytes)?;
        Ok(Self { bytes })
    }

    pub fn get_field(self, id: FieldId) -> Result<Option<ValueRef<'a>>, RecordError> {
        self.get_key(KeyRef::Field(id))
    }

    pub fn get_name(self, name: &str) -> Result<Option<ValueRef<'a>>, RecordError> {
        self.get_key(KeyRef::Name(name))
    }
    fn get_key(self, key: KeyRef<'_>) -> Result<Option<ValueRef<'a>>, RecordError> {
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

    pub fn count(self) -> Result<usize, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.map()?, 0)?;
        usize::try_from(count).map_err(|_| RecordError::SizeLimit)
    }

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
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RecordError> {
        let list = Self { bytes };
        let encoded = list.encoded_bytes()?;
        if encoded.len() != bytes.len() {
            return Err(RecordError::NotOneItem {
                offset: encoded.len(),
            });
        }
        Ok(list)
    }

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

    pub fn iter(
        self,
    ) -> Result<impl Iterator<Item = Result<ValueRef<'a>, RecordError>>, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        Ok(iter::entries(decoder, count, readers::read_value))
    }

    pub fn count(self) -> Result<usize, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        usize::try_from(count).map_err(|_| RecordError::SizeLimit)
    }

    pub fn header_len(self) -> Result<usize, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        readers::definite(decoder.array()?, 0)?;
        Ok(decoder.position())
    }

    pub fn encoded_storage(self) -> &'a [u8] {
        self.bytes
    }

    pub fn encoded_bytes(self) -> Result<&'a [u8], RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        (0..count).try_for_each(|_| skip::value(&mut decoder))?;
        self.bytes
            .get(..decoder.position())
            .ok_or(RecordError::Missing {
                offset: decoder.position(),
            })
    }

    pub fn encoded_item_offsets(self) -> Result<Vec<u32>, RecordError> {
        let mut decoder = Decoder::new(self.bytes);
        let count = readers::definite(decoder.array()?, 0)?;
        (0..count)
            .map(|_| {
                let offset =
                    u32::try_from(decoder.position()).map_err(|_| RecordError::SizeLimit)?;
                skip::value(&mut decoder)?;
                Ok(offset)
            })
            .collect()
    }
}
