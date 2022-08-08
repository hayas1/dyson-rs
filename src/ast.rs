use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

#[derive(PartialEq, Debug)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool),
    Null,
    String(String),
    Integer(i64),
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

impl Value {
    pub fn get<I: JsonIndex>(&self, index: I) -> Option<&I::Output> {
        index.gotten(self)
    }

    pub fn get_mut<I: JsonIndex>(&mut self, index: I) -> Option<&mut I::Output> {
        index.gotten_mut(self)
    }
}

pub struct Ranger<I>(I);
pub trait JsonIndex {
    type Output: ?Sized;
    fn gotten(self, value: &Value) -> Option<&Self::Output>;
    fn gotten_mut(self, value: &mut Value) -> Option<&mut Self::Output>;
    fn indexed(self, value: &Value) -> &Self::Output;
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output;
}
impl<'a> JsonIndex for &'a str {
    type Output = Value;
    fn gotten(self, value: &Value) -> Option<&Self::Output> {
        match value {
            Value::Object(m) => m.get(self),
            _ => None,
        }
    }
    fn gotten_mut(self, value: &mut Value) -> Option<&mut Self::Output> {
        match value {
            Value::Object(m) => m.get_mut(self),
            _ => None,
        }
    }
    fn indexed(self, value: &Value) -> &Self::Output {
        self.gotten(value).expect("&str index can access Object value only (or no such key)")
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        self.gotten_mut(value).expect("&str index can access Object value only (or no such key)")
    }
}
impl JsonIndex for usize {
    type Output = Value;
    fn gotten(self, value: &Value) -> Option<&Self::Output> {
        match value {
            Value::Array(v) => v.get(self),
            _ => None,
        }
    }
    fn gotten_mut(self, value: &mut Value) -> Option<&mut Self::Output> {
        match value {
            Value::Array(v) => v.get_mut(self),
            _ => None,
        }
    }
    fn indexed(self, value: &Value) -> &Self::Output {
        match value {
            Value::Array(v) => &v[self],
            _ => panic!("usize index can access Array value only"),
        }
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        match value {
            Value::Array(v) => &mut v[self],
            _ => panic!("usize index can access Array value only"),
        }
    }
}
impl<I: SliceIndex<[Value]>> JsonIndex for Ranger<I> {
    type Output = I::Output;
    fn gotten(self, value: &Value) -> Option<&Self::Output> {
        match value {
            Value::Array(v) => v.get(self.0),
            _ => None,
        }
    }
    fn gotten_mut(self, value: &mut Value) -> Option<&mut Self::Output> {
        match value {
            Value::Array(v) => v.get_mut(self.0),
            _ => None,
        }
    }
    fn indexed(self, value: &Value) -> &Self::Output {
        match value {
            Value::Array(v) => &v[self.0],
            _ => panic!("usize range index can access Array value only"),
        }
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        match value {
            Value::Array(v) => &mut v[self.0],
            _ => panic!("usize range index can access Array value only"),
        }
    }
}
impl<'a, I: JsonIndex> Index<I> for Value {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        index.indexed(self)
    }
}
impl<'a, I: JsonIndex> IndexMut<I> for Value {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.indexed_mut(self)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_access_json() {
        let json = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "parser", 1, 2, 3]"#,
            r#"}"#,
        ]
        .into_iter()
        .collect();
        let ast_root = Parser::new(&json).parse_value().unwrap();
        assert_eq!(ast_root["language"], Value::String("rust".to_string()));
        assert_eq!(ast_root["version"], Value::Float(0.1));
        assert_eq!(ast_root["keyword"][1], Value::String("json".to_string()));
        assert_eq!(
            ast_root["keyword"][Ranger(2..)],
            [Value::String("parser".to_string()), Value::Integer(1), Value::Integer(2), Value::Integer(3)]
        );
        let keyword = &ast_root["keyword"];
        assert_eq!(keyword[0], Value::String("rust".to_string()));
        assert_eq!(
            keyword[Ranger(..=2)],
            [Value::String("rust".to_string()), Value::String("json".to_string()), Value::String("parser".to_string())]
        );
        assert_eq!(keyword[Ranger(..=2)][2], Value::String("parser".to_string()));
        // compile error
        // let _ = ast_root["keyword"][Ranger(..3)]["str"]; // the type `[ast::Value]` cannot be indexed by `&str`

        // runtime error
        // let _ = &ast_root["version"][0][1]; // usize index can access Array value only
        // let _ = &ast_root["keyword"][999999999999]; // index out of bounds: the len is 6 but the index is 999999999999
    }
}
