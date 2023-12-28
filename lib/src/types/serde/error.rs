use serde::de::{Error, Expected, Unexpected as Unexp};
use std::{fmt, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    /// Raised when a `Deserialize` receives a type different from what it was
    /// expecting.
    #[error("Invalid type: {received}, expected {expected}")]
    InvalidType {
        received: Unexpected,
        expected: String,
    },

    /// Raised when a `Deserialize` receives a value of the right type but that
    /// is wrong for some other reason.
    #[error("Invalid value: {received}, expected {expected}")]
    InvalidValue {
        received: Unexpected,
        expected: String,
    },

    /// Raised when deserializing a sequence or map and the input data contains
    /// too many or too few elements.
    #[error("Invalid length {received}, expected {expected}")]
    InvalidLength { received: usize, expected: String },

    /// Raised when a `Deserialize` enum type received a variant with an
    /// unrecognized name.
    #[error("Unknown variant `{variant}`, expected {expected:?}")]
    UnknownVariant {
        variant: String,
        expected: &'static [&'static str],
    },

    /// Raised when a `Deserialize` struct type received a field with an
    /// unrecognized name.
    #[error("Unknown field `{field}`, expected {expected:?}")]
    UnknownField {
        field: String,
        expected: &'static [&'static str],
    },

    /// Raised when a `Deserialize` struct type expected to receive a required
    /// field with a particular name but that field was not present in the
    /// input.
    #[error("Missing field `{field}`")]
    MissingField { field: &'static str },

    /// Raised when a `Deserialize` struct type received more than one of the
    /// same field.
    #[error("Duplicate field `{field}`")]
    DuplicateField { field: &'static str },

    #[error("The property does not exist")]
    NoSuchProperty,

    #[error(
        "The property is missing but the deserializer still expects a value. \
        If you have an optional property with a default value, you need to \
        use an Option<T> instead (the default attribute does not work in \
        this particular instance). If you meant to extract additional data \
        other than properties, you need to use the appropriate struct wrapper."
    )]
    PropertyMissingButRequired,

    #[error("{0}")]
    Other(String),

    #[error("Could not convert the integer `{1}` to the target type {2}")]
    IntegerOutOfBounds(#[source] std::num::TryFromIntError, i64, &'static str),

    #[error("Could not convert the DateTime to the target type {0}")]
    DateTimeOutOfBounds(&'static str),
}

/// `Unexpected` represents an unexpected invocation of any one of the `Visitor`
/// trait methods.
///
/// This mirrors the [`serde::de::Unexpected`] type, but uses owned types
/// instead of borrowed types.
///
/// The owned typed in question are `String` and `Vec<u8>`, which are stored
/// as `Arc<str>` and `Arc<[u8]>` respectively, so that this type is cheap-ish
/// to clone and can be shared between threads.
#[derive(Clone, PartialEq, Debug)]
pub enum Unexpected {
    /// The input contained a boolean value that was not expected.
    Bool(bool),

    /// The input contained an unsigned integer `u8`, `u16`, `u32` or `u64` that
    /// was not expected.
    Unsigned(u64),

    /// The input contained a signed integer `i8`, `i16`, `i32` or `i64` that
    /// was not expected.
    Signed(i64),

    /// The input contained a floating point `f32` or `f64` that was not
    /// expected.
    Float(f64),

    /// The input contained a `char` that was not expected.
    Char(char),

    /// The input contained a `&str` or `String` that was not expected.
    Str(Arc<str>),

    /// The input contained a `&[u8]` or `Vec<u8>` that was not expected.
    Bytes(Arc<[u8]>),

    /// The input contained a unit `()` that was not expected.
    Unit,

    /// The input contained an `Option<T>` that was not expected.
    Option,

    /// The input contained a newtype struct that was not expected.
    NewtypeStruct,

    /// The input contained a sequence that was not expected.
    Seq,

    /// The input contained a map that was not expected.
    Map,

    /// The input contained an enum that was not expected.
    Enum,

    /// The input contained a unit variant that was not expected.
    UnitVariant,

    /// The input contained a newtype variant that was not expected.
    NewtypeVariant,

    /// The input contained a tuple variant that was not expected.
    TupleVariant,

    /// The input contained a struct variant that was not expected.
    StructVariant,

    /// A message stating what uncategorized thing the input contained that was
    /// not expected.
    ///
    /// The message should be a noun or noun phrase, not capitalized and without
    /// a period. An example message is "unoriginal superhero".
    Other(Arc<str>),
}

impl From<Unexp<'_>> for Unexpected {
    fn from(value: Unexp<'_>) -> Self {
        match value {
            Unexp::Bool(v) => Self::Bool(v),
            Unexp::Unsigned(v) => Self::Unsigned(v),
            Unexp::Signed(v) => Self::Signed(v),
            Unexp::Float(v) => Self::Float(v),
            Unexp::Char(v) => Self::Char(v),
            Unexp::Str(v) => Self::Str(v.into()),
            Unexp::Bytes(v) => Self::Bytes(v.into()),
            Unexp::Unit => Self::Unit,
            Unexp::Option => Self::Option,
            Unexp::NewtypeStruct => Self::NewtypeStruct,
            Unexp::Seq => Self::Seq,
            Unexp::Map => Self::Map,
            Unexp::Enum => Self::Enum,
            Unexp::UnitVariant => Self::UnitVariant,
            Unexp::NewtypeVariant => Self::NewtypeVariant,
            Unexp::TupleVariant => Self::TupleVariant,
            Unexp::StructVariant => Self::StructVariant,
            Unexp::Other(v) => Self::Other(v.into()),
        }
    }
}

impl Unexpected {
    fn into(&self) -> Unexp<'_> {
        match self {
            Unexpected::Bool(v) => Unexp::Bool(*v),
            Unexpected::Unsigned(v) => Unexp::Unsigned(*v),
            Unexpected::Signed(v) => Unexp::Signed(*v),
            Unexpected::Float(v) => Unexp::Float(*v),
            Unexpected::Char(v) => Unexp::Char(*v),
            Unexpected::Str(v) => Unexp::Str(v),
            Unexpected::Bytes(v) => Unexp::Bytes(v),
            Unexpected::Unit => Unexp::Unit,
            Unexpected::Option => Unexp::Option,
            Unexpected::NewtypeStruct => Unexp::NewtypeStruct,
            Unexpected::Seq => Unexp::Seq,
            Unexpected::Map => Unexp::Map,
            Unexpected::Enum => Unexp::Enum,
            Unexpected::UnitVariant => Unexp::UnitVariant,
            Unexpected::NewtypeVariant => Unexp::NewtypeVariant,
            Unexpected::TupleVariant => Unexp::TupleVariant,
            Unexpected::StructVariant => Unexp::StructVariant,
            Unexpected::Other(v) => Unexp::Other(v),
        }
    }
}

impl fmt::Display for Unexpected {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.into().fmt(formatter)
    }
}

impl Error for DeError {
    fn invalid_type(unexp: Unexp, exp: &dyn Expected) -> Self {
        Self::InvalidType {
            received: unexp.into(),
            expected: exp.to_string(),
        }
    }

    fn invalid_value(unexp: Unexp, exp: &dyn Expected) -> Self {
        Self::InvalidValue {
            received: unexp.into(),
            expected: exp.to_string(),
        }
    }

    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        Self::InvalidLength {
            received: len,
            expected: exp.to_string(),
        }
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self::UnknownVariant {
            variant: variant.to_string(),
            expected,
        }
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self::UnknownField {
            field: field.to_string(),
            expected,
        }
    }

    fn missing_field(field: &'static str) -> Self {
        Self::MissingField { field }
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::DuplicateField { field }
    }

    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Other(msg.to_string())
    }
}
