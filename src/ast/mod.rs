mod index;
mod into;

use crate::quote;
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
            Value::String(string) => string.to_string(),
            Value::Integer(integer) => integer.to_string(),
            Value::Float(float) => float.to_string(),
        }
    }
}

impl Value {
    // pub fn stringify(&self, indent: usize) -> String {
    //     let indent_unit = " ".repeat(4);
    //     let indent_internal = indent_unit.repeat(indent + 1);
    //     let indent_external = indent_unit.repeat(indent);
    //     match self {
    //         Value::Object(object) => format!(
    //             "{{\n{}\n{indent_external}}}\n",
    //             object
    //                 .iter()
    //                 .map(|(k, v)| format!("{indent_internal}{}: {}", quote(k), v.stringify(indent + 1)))
    //                 .collect::<Vec<_>>()
    //                 .join(",\n"),
    //         ),
    //         Value::Array(array) => {
    //             format!(
    //                 "[\n{}\n{indent_external}]\n",
    //                 array
    //                     .iter()
    //                     .map(|v| format!("{indent_internal}{}", v.stringify(indent + 1)))
    //                     .collect::<Vec<_>>()
    //                     .join(",\n")
    //             )
    //         }
    //         Value::Bool(bool) => bool.to_string(),
    //         Value::Null => "null".to_string(),
    //         Value::String(string) => quote(string),
    //         Value::Integer(integer) => integer.to_string(),
    //         Value::Float(float) => float.to_string(),
    //     }
    // }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_stringify_json() {
        // let json = [
        //     r#"{"#,
        //     r#"    "language": "rust","#,
        //     r#"    "notation": "json","#,
        //     r#"    "version": 0.1,"#,
        //     r#"    "keyword": ["rust", "json", "parser", 1, 2, 3],"#,
        //     r#"    "dict": {"one": 1, "two": 2, "three": 3}"#,
        //     r#"}"#,
        // ]
        // .into_iter()
        // .collect();
        // let ast_root = Parser::new(&json).parse_value().unwrap();
        // println!("{}", ast_root.stringify(0));
    }
}
