pub mod edit;
pub mod index;
pub mod into;
pub mod io;
pub mod visit;

use std::collections::HashMap;

/// [`Value`] is ast node of json. see [Introducing JSON](https://www.json.org/json-en.html) also.
/// # supports
/// - ***parser*** parse from str, file, and path. see [`Value::parse`], [`Value::read`], and [`Value::load`].
/// - ***stringify*** dump to str, file, and path. see [`Value::stringify`], [`Value::write`], and [`Value::dump`].
/// - ***indexing*** access to parsed json element (support index access). see [`index::Ranger`], [`Value::get`], and so on.
///   - and evaluate it expected type (unexpected type cause panic). see [`Value::object`] and so on.
/// - ***recombination***(developing) edit ast structure. see [`Value::swap`], [`Value::update_with`] and so on.
/// - ***visitor***(yet) iterate with dfs order. see // TODO
///
/// # examples
/// this example is read from and write to `String`.
/// read from and write to file, see [crate] and [`Value::load`], [`Value::dump`] also.
/// ```
/// use dyson::{Value, Ranger};
///
/// let raw_json = r#"
/// {
///     "language": "rust",
///     "notation": "json",
///     "version": 0.1,
///     "keyword": ["rust", "json", "parser"]
/// }"#;
/// // read json
/// let json = Value::parse(raw_json).unwrap();
///
/// // access json
/// assert_eq!(json["language"], Value::String("rust".to_string()));
/// assert_eq!(json["version"].float(), &0.1);
/// assert_eq!(json["keyword"][Ranger(1..)], [Value::String("json".to_string()), Value::String("parser".to_string())]);
/// assert_eq!(json.get("get"), None);
///
/// // edit json
/// let mut json = json;
/// json["language"] = "ruby".into();
/// assert_eq!(json["language"], Value::String("ruby".to_string()));
/// let prev_version = json["version"].swap(&mut 0.2.into());
/// assert_eq!(prev_version.float(), &0.1);
/// assert_eq!(json["version"].float(), &0.2);
/// json["keyword"].update_with(|v| v.iter().map(|k| Value::from(k.string().to_uppercase())).collect());
/// assert_eq!(json["keyword"].array(), &vec!["RUST".into(), "JSON".into(), "PARSER".into()]);
///
/// // write json
/// let str_json = json.stringify();
/// assert!(str_json.contains("\"language\""));
/// ```
#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    /// correspond to object of json. object can be represented by `HashMap` in rust.
    Object(HashMap<String, Value>),

    /// correspond to array of json. array can be represented by `Vec` in rust.
    Array(Vec<Value>),

    /// correspond to bool of json.
    Bool(bool),

    /// correspond to null of json.
    Null,

    /// correspond to string of json.
    String(String),

    /// correspond to integer of json. json has only number, but rust has integer.
    Integer(i64),

    /// correspond to float of json. json has only number, but rust has float.
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
    use super::*;
    #[test]
    fn test_stringify_json() {
        let json: String = [
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
        let ast_root = Value::parse(json).unwrap();
        let json2 = ast_root.stringify();
        let ast_root2 = Value::parse(json2).unwrap();
        let json3 = ast_root2.to_string();
        let ast_root3 = Value::parse(json3).unwrap();
        assert_eq!(ast_root, ast_root2);
        assert_eq!(ast_root2, ast_root3);
        assert_eq!(ast_root3, ast_root);
    }
}
