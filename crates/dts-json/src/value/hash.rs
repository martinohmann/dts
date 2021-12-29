use crate::Value;
use std::hash::{Hash, Hasher};

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => {
                ().hash(state);
                state.write_u8(0xF9);
            }
            Value::Bool(b) => {
                b.hash(state);
                state.write_u8(0xFA);
            }
            Value::Number(n) => {
                n.hash(state);
                state.write_u8(0xFB);
            }
            Value::String(s) => {
                s.hash(state);
                state.write_u8(0xFC);
            }
            Value::Array(array) => {
                for v in array {
                    v.hash(state);
                }
                state.write_u8(0xFD);
            }
            Value::Object(object) => {
                for (k, v) in object {
                    k.hash(state);
                    state.write_u8(0xFE);
                    v.hash(state);
                }
                state.write_u8(0xFF);
            }
        }
    }
}
