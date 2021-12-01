use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display};

/// Represents a HCL number.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Represents a positive integer.
    PosInt(u64),
    /// Represents a negative integer.
    NegInt(i64),
    /// Represents a float.
    Float(f64),
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                fn from(u: $ty) -> Self {
                    Self::PosInt(u as u64)
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                fn from(i: $ty) -> Self {
                    if i < 0 {
                        Self::NegInt(i as i64)
                    } else {
                        Self::PosInt(i as u64)
                    }
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Self::Float(f as f64)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PosInt(i) => Display::fmt(&i, formatter),
            Self::NegInt(i) => Display::fmt(&i, formatter),
            Self::Float(f) => Display::fmt(&f, formatter),
        }
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::PosInt(i) => serializer.serialize_u64(i),
            Self::NegInt(i) => serializer.serialize_i64(i),
            Self::Float(f) => serializer.serialize_f64(f),
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL number")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_f64<E>(self, value: f64) -> Result<Number, E> {
                Ok(value.into())
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}
