//! Owned values and the scalar types that can only hold canonical states.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

/// The one NaN bit pattern the format allows.
const CANONICAL_NAN: u64 = 0x7FF8_0000_0000_0000;
/// Nanoseconds in one second.
const NANOS_PER_SECOND: u32 = 1_000_000_000;

/// Numeric identifier of a structure member. Never zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(NonZeroU32);

impl FieldId {
    /// Returns the field id for `id`, or `None` for zero.
    #[must_use]
    pub const fn new(id: u32) -> Option<Self> {
        match NonZeroU32::new(id) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    /// The numeric value.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// A 64-bit float compared by its bits, with every NaN folded into one.
///
/// `0.0` and `-0.0` are different values because their bits differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Float(u64);

impl Float {
    /// The IEEE-754 bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// The floating-point value.
    #[must_use]
    pub const fn get(self) -> f64 {
        f64::from_bits(self.0)
    }

    /// Accepts stored bits only when they are a canonical encoding.
    pub(super) fn from_canonical_bits(bits: u64) -> Option<Self> {
        (!f64::from_bits(bits).is_nan() || bits == CANONICAL_NAN).then_some(Self(bits))
    }
}

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        if value.is_nan() {
            Self(CANONICAL_NAN)
        } else {
            Self(value.to_bits())
        }
    }
}

/// A point in time: seconds from the Unix epoch and nanoseconds below one second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    seconds: i64,
    nanos: u32,
}

impl Time {
    /// Returns the time, or `None` when `nanos` is one second or more.
    #[must_use]
    pub const fn new(seconds: i64, nanos: u32) -> Option<Self> {
        if nanos < NANOS_PER_SECOND {
            Some(Self { seconds, nanos })
        } else {
            None
        }
    }

    /// Whole seconds from the Unix epoch.
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.seconds
    }

    /// Nanoseconds within the second.
    #[must_use]
    pub const fn nanos(self) -> u32 {
        self.nanos
    }
}

/// An owned value tree, the input to [`Record::encode`](super::Record::encode).
///
/// Maps and structs are ordered collections, so two equal values always
/// encode to the same bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An explicit null, as in a sparse list.
    Null,
    /// A boolean.
    Bool(bool),
    /// A signed integer of any modeled width.
    Int(i64),
    /// A float or double.
    Float(Float),
    /// A string, enum name, or arbitrary-precision number in text form.
    Text(String),
    /// A blob.
    Bytes(Vec<u8>),
    /// A timestamp.
    Time(Time),
    /// A list.
    List(Vec<Value>),
    /// A map with string keys.
    Map(BTreeMap<String, Value>),
    /// A structure or union, keyed by member id.
    Struct(BTreeMap<FieldId, Value>),
}
