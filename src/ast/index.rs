use super::Value;
use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl Value {
    /// access json value. get reference of it.
    /// - if index is position and the value is array, return the element, else return `None`
    /// - if index is range and the value is array, return these element, else return `None`
    /// - if index is string and the value is object, return the element, else return `None`
    pub fn get<I: JsonIndex>(&self, index: I) -> Option<&I::Output> {
        index.gotten(self)
    }

    /// access json value. get mutable reference of it. see [get](Value) also.
    pub fn get_mut<I: JsonIndex>(&mut self, index: I) -> Option<&mut I::Output> {
        index.gotten_mut(self)
    }
}

pub struct Ranger<R>(pub R);
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
        match value {
            Value::Object(m) => &m[self],
            _ => panic!("&str index can access Object value only, but {}", value.node_type()),
        }
    }
    fn indexed_mut(self, _value: &mut Value) -> &mut Self::Output {
        // match value {
        //     Value::Object(_) => self.gotten_mut(value).unwrap_or_else(|| panic!("no such key: \"{self}\"")),
        //     _ => panic!("&str index can access Object value only, but {}", value.node_type()),
        // }
        unimplemented!("HashMap do not implement IndexMut")
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
            _ => panic!("usize index can access Array value only, but {}", value.node_type()),
        }
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        match value {
            Value::Array(v) => &mut v[self],
            _ => panic!("usize index can access Array value only, but {}", value.node_type()),
        }
    }
}
impl<R: SliceIndex<[Value]>> JsonIndex for Ranger<R> {
    type Output = R::Output;
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
            _ => panic!("usize range index can access Array value only, but {}", value.node_type()),
        }
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        match value {
            Value::Array(v) => &mut v[self.0],
            _ => panic!("usize range index can access Array value only, but {}", value.node_type()),
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
