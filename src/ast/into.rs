use super::Value;
use indexmap::IndexMap;

/// evaluate `Value` to corresponded object such as `IndexMap`, `Vec`, `bool`, `str`, `i64`, or `f64`.
/// # panics
/// call different type evaluate method cause panic.
/// for example, if call [`Value::object`] to [`Value::Array`], it will panic.
/// if want to get `None` instead of panic, use `get_` prefixed methods.
impl Value {
    pub fn get_object(&self) -> Option<&IndexMap<String, Value>> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }
    pub fn get_mut_object(&mut self) -> Option<&mut IndexMap<String, Value>> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }
    pub fn object(&self) -> &IndexMap<String, Value> {
        self.get_object().unwrap_or_else(|| panic!("only Object can convert into IndexMap, but {}", self.node_type()))
    }

    pub fn get_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }
    pub fn get_mut_array(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(m) => Some(m),
            _ => None,
        }
    }
    pub fn array(&self) -> &Vec<Value> {
        self.get_array().unwrap_or_else(|| panic!("only Array can convert into Vec, but {}", self.node_type()))
    }

    pub fn get_bool(&self) -> Option<&bool> {
        match self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
    pub fn get_mut_bool(&mut self) -> Option<&mut bool> {
        match self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
    pub fn bool(&self) -> &bool {
        self.get_bool().unwrap_or_else(|| panic!("only Bool can convert into bool, but {}", self.node_type()))
    }

    pub fn get_null(&self) -> Option<()> {
        match self {
            Value::Null => Some(()),
            _ => None,
        }
    }
    pub fn null(&self) {
        self.get_null().unwrap_or_else(|| panic!("only Null can convert into null, but {}", self.node_type()))
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn get_mut_string(&mut self) -> Option<&mut str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn string(&self) -> &str {
        self.get_string().unwrap_or_else(|| panic!("only String can convert into &str, but {}", self.node_type()))
    }

    pub fn get_integer(&self) -> Option<&i64> {
        match self {
            Value::Integer(i) => Some(i),
            _ => None,
        }
    }
    pub fn get_mut_integer(&mut self) -> Option<&mut i64> {
        match self {
            Value::Integer(i) => Some(i),
            _ => None,
        }
    }
    pub fn integer(&self) -> &i64 {
        self.get_integer().unwrap_or_else(|| panic!("only Integer can convert into i64, but {}", self.node_type()))
    }

    pub fn get_float(&self) -> Option<&f64> {
        match self {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }
    pub fn get_mut_float(&mut self) -> Option<&mut f64> {
        match self {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }
    pub fn float(&self) -> &f64 {
        self.get_float().unwrap_or_else(|| panic!("only Float can convert into f64, but {}", self.node_type()))
    }
}

impl Value {
    /// iterate [`Value::Object`]
    /// # panics
    /// if value is not `Object`.
    /// # examples
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{"foo": [1, "2", 3, "4", 5], "bar": 6}"#;
    /// let mut json = Value::parse(raw_json).unwrap();
    ///
    /// use std::collections::HashSet;
    /// assert_eq!(
    ///     json.items().map(|(k, _v)| &k[..]).collect::<HashSet<_>>(),
    ///     vec!["foo", "bar"].into_iter().collect()
    /// );
    /// ```
    pub fn items(&self) -> impl Iterator<Item = (&String, &Value)> {
        match self {
            Value::Object(m) => m.iter(),
            _ => panic!("only Object can iterate with items, but {}", self.node_type()),
        }
    }
    /// iterate [`Value::Array`]
    /// # panics
    /// if value is not `Array`.
    /// # examples
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{"foo": [1, "two", 3], "bar": 6}"#;
    /// let mut json = Value::parse(raw_json).unwrap();
    ///
    /// assert_eq!(json["foo"].iter().map(|(v)| v.to_string()).collect::<Vec<_>>(), vec!["1", "\"two\"", "3"]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        match self {
            Value::Array(v) => v.iter(),
            _ => panic!("only Array can iterate, but {}", self.node_type()),
        }
    }
}

impl From<Value> for IndexMap<String, Value> {
    fn from(val: Value) -> Self {
        match val {
            Value::Object(m) => m,
            _ => panic!("only Object can convert into IndexMap, but {}", val.node_type()),
        }
    }
}
impl<'a> From<&'a Value> for &'a IndexMap<String, Value> {
    fn from(val: &'a Value) -> Self {
        match val {
            Value::Object(m) => m,
            _ => panic!("only Object can convert into IndexMap, but {}", val.node_type()),
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

/// check node type methods.
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

impl From<IndexMap<String, Value>> for Value {
    fn from(m: IndexMap<String, Value>) -> Self {
        Value::Object(m)
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}
impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl FromIterator<(String, Value)> for Value {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Value::Object(iter.into_iter().collect())
    }
}
impl FromIterator<Value> for Value {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Value::Array(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::Parser;

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
