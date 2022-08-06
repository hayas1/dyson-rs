use std::collections::HashMap;

pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool),
    Null,
    String(String),
    Integer(u64),
    Float(f64),
}
