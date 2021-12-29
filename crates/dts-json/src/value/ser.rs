use super::{to_value, Map, Number, Value};
use crate::{Error, Result};
use serde::ser::{self, Impossible, Serialize};
use std::fmt;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::Number(ref n) => n.serialize(serializer),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Array(ref v) => v.serialize(serializer),
            Value::Object(ref v) => v.serialize(serializer),
        }
    }
}

pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, value: bool) -> Result<Value> {
        Ok(Value::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Value> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(self, value: i16) -> Result<Value> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i32(self, value: i32) -> Result<Value> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_u8(self, value: u8) -> Result<Value> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(self, value: u16) -> Result<Value> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u32(self, value: u32) -> Result<Value> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u64(self, value: u64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_f32(self, value: f32) -> Result<Value> {
        self.serialize_f64(value as f64)
    }

    fn serialize_f64(self, value: f64) -> Result<Value> {
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    fn serialize_char(self, value: char) -> Result<Value> {
        let mut s = String::new();
        s.push(value);
        Ok(Value::String(s))
    }

    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        let vec = value.iter().map(|&b| Value::Number(b.into())).collect();
        Ok(Value::Array(vec))
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        let mut values = Map::new();
        values.insert(String::from(variant), to_value(&value)?);
        Ok(Value::Object(values))
    }

    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap {
            map: Map::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Map::new(),
        })
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Value>
    where
        T: fmt::Display,
    {
        Ok(Value::String(value.to_string()))
    }
}

pub struct SerializeVec {
    vec: Vec<Value>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

pub struct SerializeMap {
    map: Map<String, Value>,
    next_key: Option<String>,
}

pub struct SerializeStructVariant {
    name: String,
    map: Map<String, Value>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.name, Value::Array(self.vec));
        Ok(Value::Object(object))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take();
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(String::from(key), to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.name, Value::Object(self.map));
        Ok(Value::Object(object))
    }
}

fn string_key_expected() -> Error {
    Error::new("key must be a string")
}

struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_f64(self, _value: f64) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_char(self, value: char) -> Result<String> {
        Ok(String::from(value))
    }

    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(string_key_expected())
    }

    fn serialize_none(self) -> Result<String> {
        Err(string_key_expected())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(string_key_expected())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(string_key_expected())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(string_key_expected())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(string_key_expected())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(string_key_expected())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(string_key_expected())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(string_key_expected())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(string_key_expected())
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<String>
    where
        T: fmt::Display,
    {
        Ok(value.to_string())
    }
}
