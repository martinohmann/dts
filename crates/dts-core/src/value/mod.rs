//! The Value enum, a loosely typed way of representing any valid value.

mod de;
mod ext;
mod from;
mod ser;

use crate::value::ser::Serializer;
use crate::Result;
use serde::ser::Serialize;
use std::fmt;
use std::io;
use std::str;

pub use crate::number::Number;

/// The map type used for objects.
pub type Map<K, V> = indexmap::IndexMap<K, V>;

/// Represents any valid value.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Represents a null value.
    Null,
    /// Represents a boolean.
    Bool(bool),
    /// Represents a number, either integer or float.
    Number(Number),
    /// Represents a string.
    String(String),
    /// Represents a array.
    Array(Vec<Value>),
    /// Represents a object.
    Object(Map<String, Value>),
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

impl Value {
    /// If the `Value` is an Array, returns the associated vector. Returns None
    /// otherwise.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    /// If the `Value` is a Boolean, represent it as bool if possible. Returns
    /// None otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// If the `Value` is a Number, represent it as f64 if possible. Returns
    /// None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        self.as_number().and_then(|n| n.as_f64())
    }

    /// If the `Value` is a Number, represent it as i64 if possible. Returns
    /// None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_number().and_then(|n| n.as_i64())
    }

    /// If the `Value` is a Null, returns (). Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    /// If the `Value` is a Number, returns the associated Number. Returns None
    /// otherwise.
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated Map. Returns None
    /// otherwise.
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    /// If the `Value` is a String, returns the associated str. Returns None
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the `Value` is a Number, represent it as u64 if possible. Returns
    /// None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_number().and_then(|n| n.as_u64())
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    ///
    /// For any Value on which `is_array` returns true, `as_array` and
    /// `as_array_mut` are guaranteed to return the vector representing the
    /// array.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// For any Value on which `is_boolean` returns true, `as_bool` is
    /// guaranteed to return the boolean value.
    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    ///
    /// For any Value on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    pub fn is_f64(&self) -> bool {
        self.as_number().map(Number::is_f64).unwrap_or(false)
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    pub fn is_i64(&self) -> bool {
        self.as_number().map(Number::is_i64).unwrap_or(false)
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    ///
    /// For any Value on which `is_null` returns true, `as_null` is guaranteed
    /// to return `Some(())`.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// Returns true if the `Value` is an Object. Returns false otherwise.
    ///
    /// For any Value on which `is_object` returns true, `as_object` and
    /// `as_object_mut` are guaranteed to return the map representation of the
    /// object.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// For any Value on which `is_string` returns true, `as_str` is guaranteed
    /// to return the string slice.
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// Returns true if the `Value` is an integer between zero and `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    pub fn is_u64(&self) -> bool {
        self.as_number().map(Number::is_u64).unwrap_or(false)
    }

    /// Takes the value out of the `Value`, leaving a `Null` in its place.
    pub fn take(&mut self) -> Value {
        std::mem::replace(self, Value::Null)
    }
}

impl fmt::Display for Value {
    /// Display a JSON value as a string.
    ///
    /// ```
    /// # use dts_core::json;
    /// #
    /// let json = json!({ "city": "London", "street": "10 Downing Street" });
    ///
    /// // Compact format:
    /// //
    /// // {"city":"London","street":"10 Downing Street"}
    /// let compact = format!("{}", json);
    /// assert_eq!(compact,
    ///     "{\"city\":\"London\",\"street\":\"10 Downing Street\"}");
    ///
    /// // Pretty format:
    /// //
    /// // {
    /// //   "city": "London",
    /// //   "street": "10 Downing Street"
    /// // }
    /// let pretty = format!("{:#}", json);
    /// assert_eq!(pretty,
    ///     "{\n  \"city\": \"London\",\n  \"street\": \"10 Downing Street\"\n}");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct WriterFormatter<'a, 'b: 'a> {
            inner: &'a mut fmt::Formatter<'b>,
        }

        impl<'a, 'b> io::Write for WriterFormatter<'a, 'b> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                // Safety: the serializer below only emits valid utf8 when using
                // the default formatter.
                let s = unsafe { str::from_utf8_unchecked(buf) };
                self.inner.write_str(s).map_err(io_error)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        fn io_error(_: fmt::Error) -> io::Error {
            // Error value does not matter because Display impl just maps it
            // back to fmt::Error.
            io::Error::new(io::ErrorKind::Other, "fmt error")
        }

        let alternate = f.alternate();
        let mut wr = WriterFormatter { inner: f };
        if alternate {
            // {:#}
            serde_json::to_writer_pretty(&mut wr, self).map_err(|_| fmt::Error)
        } else {
            // {}
            serde_json::to_writer(&mut wr, self).map_err(|_| fmt::Error)
        }
    }
}

/// Convert a `T` into `dts_core::Value` which is an enum that can represent
/// any valid JSON data.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use dts_core::json;
///
/// use std::error::Error;
///
/// #[derive(Serialize)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn compare_json_values() -> Result<(), Box<dyn Error>> {
///     let u = User {
///         fingerprint: "0xF9BA143B95FF6D82".to_owned(),
///         location: "Menlo Park, CA".to_owned(),
///     };
///
///     // The type of `expected` is `dts_core::Value`
///     let expected = json!({
///         "fingerprint": "0xF9BA143B95FF6D82",
///         "location": "Menlo Park, CA",
///     });
///
///     let v = dts_core::to_value(u).unwrap();
///     assert_eq!(v, expected);
///
///     Ok(())
/// }
/// #
/// # compare_json_values().unwrap();
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
///
/// ```
/// use std::collections::BTreeMap;
///
/// // The keys in this map are vectors, not strings.
/// let mut map = BTreeMap::new();
/// map.insert(vec![32, 64], "x86");
///
/// println!("{}", dts_core::to_value(map).unwrap_err());
/// ```
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(Serializer)
}
