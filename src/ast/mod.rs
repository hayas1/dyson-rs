pub mod index;
pub mod into;
pub mod io;

use std::collections::HashMap;

/// `Value` is ast node of json. see [Introducing JSON](https://www.json.org/json-en.html) also.
/// # Examples
/// ``` no_run
/// use dyson::{Value, Ranger};
///
/// // `path/to/read.json`
/// // {
/// //     "language": "rust",
/// //     "notation": "json",
/// //     "version": 0.1,
/// //     "keyword": ["rust", "json", "parser"]
/// // }
/// // read json
/// let json = Value::parse_path("path/to/read.json").expect("invalid path or json structure");
///
/// // access json
/// assert_eq!(json["language"], Value::String("rust".to_string()));
/// assert_eq!(json["version"].evaluate_float(), &0.1);
/// assert_eq!(json["keyword"][Ranger(1..)], [Value::String("json".to_string()), Value::String("parser".to_string())]);
/// assert_eq!(json.get("get"), None);
///
/// // write json
/// json.stringify_path("path/to/write.json").expect("failed to write json");
/// ```
#[derive(PartialEq, Debug)]
pub enum Value {
    /// correspond to object of json.
    Object(HashMap<String, Value>),

    /// correspond to array of json.
    Array(Vec<Value>),

    /// correspond to bool of json.
    Bool(bool),

    /// correspond to null of json.
    Null,

    /// correspond to string of json.
    String(String),

    /// correspond to integer of json.
    Integer(i64),

    /// correspond to float of json.
    Float(f64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json_display = match self {
            Value::Object(object) => format!(
                "{{{}}}",
                object.iter().map(|(k, v)| format!("{}:{}", quote(k), v)).collect::<Vec<_>>().join(","),
            ),
            Value::Array(array) => {
                format!("[{}]", array.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))
            }
            Value::Bool(bool) => bool.to_string(),
            Value::Null => "null".to_string(),
            Value::String(string) => quote(string),
            Value::Integer(integer) => integer.to_string(),
            Value::Float(float) => float.to_string(),
        };
        write!(f, "{json_display}")
    }
}

impl Value {
    /// stringify ast with proper indent.
    pub fn stringify(&self) -> String {
        fn stringify_recursive(value: &Value, indent: usize) -> String {
            let indent_unit = " ".repeat(4);
            let indent_internal = indent_unit.repeat(indent + 1);
            let indent_external = indent_unit.repeat(indent);
            match value {
                Value::Object(object) => format!(
                    "{{\n{}\n{indent_external}}}",
                    object
                        .iter()
                        .map(|(k, v)| format!("{indent_internal}{}: {}", quote(k), stringify_recursive(v, indent + 1)))
                        .collect::<Vec<_>>()
                        .join(",\n"),
                ),
                Value::Array(array) => {
                    format!(
                        "[\n{}\n{indent_external}]",
                        array
                            .iter()
                            .map(|v| format!("{indent_internal}{}", stringify_recursive(v, indent + 1)))
                            .collect::<Vec<_>>()
                            .join(",\n")
                    )
                }
                Value::Bool(bool) => bool.to_string(),
                Value::Null => "null".to_string(),
                Value::String(string) => quote(string),
                Value::Integer(integer) => integer.to_string(),
                Value::Float(float) => float.to_string(),
            }
        }
        stringify_recursive(self, 0)
    }

    /// get ast node type as `&str`. mainly for debugging purposes.
    pub fn node_type(&self) -> &str {
        match self {
            Value::Object(_) => "Object",
            Value::Array(_) => "Array",
            Value::Bool(_) => "Bool",
            Value::Null => "Null",
            Value::String(_) => "String",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
        }
    }
}

fn quote(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('/', "\\/")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_stringify_json() {
        let json = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "parser", 1, 2, 3],"#,
            r#"    "dict": {"one": 1, "two": 2, "three": 3}"#,
            r#"}"#,
        ]
        .into_iter()
        .collect();
        let ast_root = Parser::new(&json).parse_value().unwrap();
        let json2 = ast_root.stringify().into();
        let ast_root2 = Parser::new(&json2).parse_value().unwrap();
        let json3 = ast_root2.to_string().into();
        let ast_root3 = Parser::new(&json3).parse_value().unwrap();
        assert_eq!(ast_root, ast_root2);
        assert_eq!(ast_root2, ast_root3);
        assert_eq!(ast_root, ast_root3);
    }
}
