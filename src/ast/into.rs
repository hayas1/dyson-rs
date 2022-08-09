use super::Value;
use std::collections::HashMap;

impl Value {
    /// evaluate `Object` node as `HashMap`
    pub fn evaluate_object(&self) -> HashMap<&str, &Value> {
        match self {
            Value::Object(m) => m.iter().map(|(k, v)| (&k[..], v)).collect(),
            _ => panic!("only Object can convert into HashMap, but {}", self.node_type()),
        }
    }
    /// evaluate `Array` node as `Vec`
    pub fn evaluate_array(&self) -> Vec<&Value> {
        match self {
            Value::Array(v) => v.iter().collect(),
            _ => panic!("only Array can convert into Vec, but {}", self.node_type()),
        }
    }
    /// evaluate `Bool` node as `bool`
    pub fn evaluate_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => panic!("only Bool can convert into bool, but {}", self.node_type()),
        }
    }
    /// evaluate `String` node as `String`
    pub fn evaluate_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            _ => panic!("only String can convert into String, but {}", self.node_type()),
        }
    }
    /// evaluate `Integer` node as `i64`
    pub fn evaluate_integer(&self) -> i64 {
        match self {
            Value::Integer(i) => *i,
            _ => panic!("only Integer can convert into i64, but {}", self.node_type()),
        }
    }
    /// evaluate `Float` node as `f64`
    pub fn evaluate_float(&self) -> f64 {
        match self {
            Value::Float(f) => *f,
            _ => panic!("only Float can convert into f64, but {}", self.node_type()),
        }
    }
}

impl From<Value> for HashMap<String, Value> {
    fn from(val: Value) -> Self {
        match val {
            Value::Object(m) => m,
            _ => panic!("only Object can convert into HashMap, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a HashMap<String, Value> {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Object(m) => m,
            _ => panic!("only Object can convert into HashMap, but {}", val.node_type()),
        }
    }
}

impl From<Value> for Vec<Value> {
    fn from(val: Value) -> Self {
        match val {
            Value::Array(v) => v,
            _ => panic!("only Array can convert into Vec, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a Vec<Value> {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Array(v) => v,
            _ => panic!("only Array can convert into Vec, but {}", val.node_type()),
        }
    }
}

impl From<Value> for bool {
    fn from(val: Value) -> Self {
        match val {
            Value::Bool(b) => b,
            _ => panic!("only Bool can convert into bool, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a bool {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Bool(b) => b,
            _ => panic!("only Bool can convert into bool, but {}", val.node_type()),
        }
    }
}

impl From<Value> for String {
    fn from(val: Value) -> Self {
        match val {
            Value::String(s) => s,
            _ => panic!("only String can convert into String, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a str {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::String(s) => s,
            _ => panic!("only String can convert into String, but {}", val.node_type()),
        }
    }
}

impl From<Value> for i64 {
    fn from(val: Value) -> Self {
        match val {
            Value::Integer(i) => i,
            _ => panic!("only Integer can convert into i64, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a i64 {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Integer(i) => i,
            _ => panic!("only Integer can convert into i64, but {}", val.node_type()),
        }
    }
}

impl From<Value> for f64 {
    fn from(val: Value) -> Self {
        match val {
            Value::Float(f) => f,
            _ => panic!("only Float can convert into f64, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a f64 {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Float(f) => f,
            _ => panic!("only Float can convert into f64, but {}", val.node_type()),
        }
    }
}

impl Value {
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    pub fn is_true(&self) -> bool {
        matches!(self, Value::Bool(true))
    }
    pub fn is_false(&self) -> bool {
        matches!(self, Value::Bool(false))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_into_bool_json() {
        let tru = "true".into();
        let tru_ast = Parser::new(&tru).parse_bool().unwrap();
        let t: &bool = (&tru_ast).into();
        assert_eq!(t, &true);
        let t: bool = tru_ast.into();
        assert!(t);
    }

    #[test]
    fn test_into_string_json() {
        let string = "\"rust\"".into();
        let string_ast = Parser::new(&string).parse_string().unwrap();
        let s: &str = (&string_ast).into();
        assert_eq!(s, "rust");
        let s: String = string_ast.into();
        assert_eq!(s, "rust".to_string());
    }

    #[test]
    fn test_into_integer_json() {
        let hundred = "100".into();
        let hundred_ast = Parser::new(&hundred).parse_number().unwrap();
        let i: &i64 = (&hundred_ast).into();
        assert_eq!(i, &100);
        let i: i64 = hundred_ast.into();
        assert_eq!(i, 100);
    }

    #[test]
    fn test_into_float_json() {
        let quarter = "0.25".into();
        let quarter_ast = Parser::new(&quarter).parse_number().unwrap();
        let f: &f64 = (&quarter_ast).into();
        assert_eq!(f, &0.25);
        let f: f64 = quarter_ast.into();
        assert_eq!(f, 0.25);
    }
}
