mod index;
mod into;

use std::collections::HashMap;

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
                object.iter().map(|(k, v)| format!("{}:{}", quote(k), v.to_string())).collect::<Vec<_>>().join(","),
            ),
            Value::Array(array) => {
                format!("[{}]", array.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))
            }
            Value::Bool(bool) => bool.to_string(),
            Value::Null => "null".to_string(),
            Value::String(string) => quote(string),
            Value::Integer(integer) => integer.to_string(),
            Value::Float(float) => float.to_string(),
        }
    }
}

impl Value {
    pub fn stringify(&self, indent: usize) -> String {
        let indent_unit = " ".repeat(4);
        let indent_internal = indent_unit.repeat(indent + 1);
        let indent_external = indent_unit.repeat(indent);
        match self {
            Value::Object(object) => format!(
                "{{\n{}\n{indent_external}}}",
                object
                    .iter()
                    .map(|(k, v)| format!("{indent_internal}{}: {}", quote(k), v.stringify(indent + 1)))
                    .collect::<Vec<_>>()
                    .join(",\n"),
            ),
            Value::Array(array) => {
                format!(
                    "[\n{}\n{indent_external}]",
                    array
                        .iter()
                        .map(|v| format!("{indent_internal}{}", v.stringify(indent + 1)))
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
        let json2 = ast_root.stringify(0).into();
        let ast_root2 = Parser::new(&json2).parse_value().unwrap();
        let json3 = ast_root2.to_string().into();
        let ast_root3 = Parser::new(&json3).parse_value().unwrap();
        assert_eq!(ast_root, ast_root2);
        assert_eq!(ast_root2, ast_root3);
        assert_eq!(ast_root, ast_root3);
    }
}
