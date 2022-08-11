use super::Value;
use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl Value {
    /// access json value, and get reference of it. see indexing [`Ranger`] also.
    /// - if value is array
    ///   - if index is position, return the element, else return `None`
    ///   - if index is range, return these elements, else return `None`
    /// - if value is object
    ///   - index is string, return the element, else return `None`
    /// # examples
    /// ```
    /// use dyson::{Ranger, Value};
    /// let raw_json = r#"{"foo": [1, "two", 3], "bar": 4}"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// let foo = json.get("foo");
    /// assert_eq!(foo, Some(&Value::Array(vec![Value::Integer(1), Value::String("two".to_string()), Value::Integer(3)])));
    /// assert_eq!(foo.unwrap().get(Ranger(..=1)), Some(&[Value::Integer(1), Value::String("two".to_string())][..]));
    /// assert_eq!(foo.unwrap().get("bar"), None);
    ///
    /// assert_eq!(json.get("bar"), Some(&Value::Integer(4)));
    /// assert_eq!(json.get("baz"), None);
    /// ```
    pub fn get<I: JsonIndex>(&self, index: I) -> Option<&I::Output> {
        index.gotten(self)
    }

    /// access json value. get mutable reference of it. see [`Value::get`] also.
    pub fn get_mut<I: JsonIndex>(&mut self, index: I) -> Option<&mut I::Output> {
        index.gotten_mut(self)
    }
}

/// [`Ranger`] is used for accessing [`Value`] by range operator. see [`Value::get`] also.
/// # examples
/// ```
/// use dyson::{Ranger, Value};
/// let raw_json = r#"{"key": [1, "two", 3, "four", 5]}"#;
/// let json = Value::parse(raw_json).unwrap();
///
/// assert_eq!(json["key"][Ranger(..2)], vec![Value::Integer(1), Value::String("two".to_string())]);
/// ```
pub struct Ranger<R>(
    /// range object like `start..end`, `..end`, `start..=end`, and so on.
    pub R,
);
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

    #[test]
    fn test_access_json() {
        let json = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "parser", 1, 2, 3]"#,
            r#"}"#,
        ];
        let ast_root = Value::parse(json.into_iter().collect::<String>()).unwrap();
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
