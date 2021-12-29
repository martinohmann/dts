use super::{Map, Number, Value};
use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::borrow::Cow;

macro_rules! impl_from_integer {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(n.into())
                }
            }
        )*
    };
}

impl_from_integer!(i8, i16, i32, i64, isize);
impl_from_integer!(u8, u16, u32, u64, usize);

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Value::Null, Value::Number)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    fn from(s: Cow<'a, str>) -> Self {
        Value::String(s.into_owned())
    }
}

impl From<Map<String, Value>> for Value {
    fn from(f: Map<String, Value>) -> Self {
        Value::Object(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(f: Vec<T>) -> Self {
        Value::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    fn from(f: &'a [T]) -> Self {
        Value::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Null
    }
}

impl From<JsonValue> for Value {
    fn from(v: JsonValue) -> Self {
        match v {
            JsonValue::Null => Value::Null,
            JsonValue::Bool(b) => Value::Bool(b),
            JsonValue::Number(n) => Number::try_from(n).map_or(Value::Null, Value::Number),
            JsonValue::String(s) => Value::String(s),
            JsonValue::Array(a) => Value::from_iter(a),
            JsonValue::Object(o) => Value::from_iter(o),
        }
    }
}

impl From<Value> for JsonValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(b),
            Value::Number(n) => JsonNumber::try_from(n).map_or(JsonValue::Null, JsonValue::Number),
            Value::String(s) => JsonValue::String(s),
            Value::Array(a) => JsonValue::from_iter(a),
            Value::Object(o) => JsonValue::from_iter(o),
        }
    }
}
