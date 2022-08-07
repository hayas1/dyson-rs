use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool),
    Null,
    String(String),
    Integer(u64),
    Float(f64),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Object(object) => format!(
                "{{{}}}",
                object.iter().map(|(k, v)| format!("\"{k}\": {}", v.to_string())).collect::<Vec<_>>().join(", "),
            ),
            Value::Array(array) => {
                format!("[{}]", array.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
            }
            Value::Bool(bool) => bool.to_string(),
            Value::Null => "null".to_string(),
            Value::String(string) => string.to_string(),
            Value::Integer(integer) => integer.to_string(),
            Value::Float(float) => float.to_string(),
        }
    }
}
