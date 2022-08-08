use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{ast::Value, json::RawJson, parser::Parser};

struct Buf<B>(B);
pub trait IntoJson {
    fn into_json(self) -> anyhow::Result<RawJson>;
}
impl<B: BufRead> IntoJson for Buf<B> {
    fn into_json(self) -> anyhow::Result<RawJson> {
        let mut json = Vec::<String>::new();
        for line in self.0.lines() {
            json.push(line?.chars().collect())
        }
        Ok(json.into_iter().collect())
    }
}
impl<'a> IntoJson for &'a Path {
    fn into_json(self) -> anyhow::Result<RawJson> {
        let file = File::open(&self)?;
        Buf(BufReader::new(file)).into_json()
    }
}
impl IntoJson for File {
    fn into_json(self) -> anyhow::Result<RawJson> {
        Buf(BufReader::new(self)).into_json()
    }
}

pub fn parse<T: Into<RawJson>>(t: T) -> anyhow::Result<Value> {
    let json = t.into();
    Parser::new(&json).parse_value()
}

pub fn parse_buf<T: IntoJson>(t: T) -> anyhow::Result<Value> {
    let json = t.into_json()?;
    Parser::new(&json).parse_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_json() {
        let s = r#"{"this": "is", "json": "parser"}"#;
        let ast_root = parse(s);
        match ast_root {
            Ok(r) => assert_eq!(r["json"], Value::String("parser".to_string())),
            Err(_) => unreachable!("string must be parsed as json"),
        }
    }

    #[test]
    fn test_string_to_json() {
        let string = r#"{"this": "is", "json": "parser"}"#.to_string();
        let ast_root = parse(string);
        match ast_root {
            Ok(r) => assert_eq!(r["json"], Value::String("parser".to_string())),
            Err(_) => unreachable!("string must be parsed as json"),
        }
    }

    #[test]
    fn test_path_to_json() {
        let path = Path::new("test/simple.json");
        let ast_root = parse_buf(path);
        match ast_root {
            Ok(r) => assert_eq!(r["language"], Value::String("Rust".to_string())),
            Err(e) => assert!(e.to_string().to_lowercase().contains("no")),
        }
    }

    #[test]
    fn test_file_to_json() {
        let file = File::open("test/simple.json");
        match file {
            Ok(f) => {
                let ast_root = parse_buf(f);
                match ast_root {
                    Ok(r) => assert_eq!(r["language"], Value::String("Rust".to_string())),
                    Err(e) => assert!(e.to_string().to_lowercase().contains("no")),
                }
            }
            Err(e) => assert!(e.to_string().to_lowercase().contains("no")),
        }
    }
}
