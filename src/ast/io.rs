use crate::{parser::Parser, rawjson::RawJson};

use super::Value;
use anyhow::Context as _;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

impl Value {
    /// parse raw json into ast.
    pub fn parse<T: Into<RawJson>>(t: T) -> anyhow::Result<Value> {
        let json = t.into();
        Parser::new(&json).parse_value()
    }
    /// parse file like raw json into ast. see [parse](Value) also.
    pub fn parse_read<T: ReadJson>(t: T) -> anyhow::Result<Value> {
        let json = t.read_json()?;
        Parser::new(&json).parse_value()
    }
    /// write ast to file. if `min` is true, no unnecessary space and linefeed is included.
    pub fn stringify_write<T: WriteJson>(&self, writer: T, min: bool) -> anyhow::Result<usize> {
        writer.write_json(&if min { self.to_string() } else { self.stringify() })
    }
}

struct Buf<B>(B);
pub trait ReadJson {
    fn read_json(self) -> anyhow::Result<RawJson>;
}
impl<B: BufRead> ReadJson for Buf<B> {
    fn read_json(self) -> anyhow::Result<RawJson> {
        let mut json = Vec::<String>::new();
        for line in self.0.lines() {
            json.push(line?.chars().collect())
        }
        Ok(json.into_iter().collect())
    }
}
impl ReadJson for File {
    fn read_json(self) -> anyhow::Result<RawJson> {
        Buf(BufReader::new(self)).read_json()
    }
}
impl<'a> ReadJson for &'a File {
    fn read_json(self) -> anyhow::Result<RawJson> {
        Buf(BufReader::new(self)).read_json()
    }
}
impl<'a> ReadJson for &'a Path {
    fn read_json(self) -> anyhow::Result<RawJson> {
        File::open(&self)?.read_json()
    }
}
impl<'a> ReadJson for &'a PathBuf {
    fn read_json(self) -> anyhow::Result<RawJson> {
        File::open(&self)?.read_json()
    }
}

pub trait WriteJson {
    fn write_json(self, json: &str) -> anyhow::Result<usize>;
}
impl<'a, T: Write> WriteJson for Buf<&'a mut BufWriter<T>> {
    fn write_json(self, json: &str) -> anyhow::Result<usize> {
        self.0.write(json.as_bytes()).context("file write error")
    }
}
impl WriteJson for File {
    fn write_json(self, json: &str) -> anyhow::Result<usize> {
        Buf(&mut BufWriter::new(self)).write_json(json)
    }
}
impl<'a> WriteJson for &'a File {
    fn write_json(self, json: &str) -> anyhow::Result<usize> {
        Buf(&mut BufWriter::new(self)).write_json(json)
    }
}
impl<'a> WriteJson for &'a Path {
    fn write_json(self, json: &str) -> anyhow::Result<usize> {
        File::create(&self)?.write_json(json)
    }
}
impl<'a> WriteJson for &'a PathBuf {
    fn write_json(self, json: &str) -> anyhow::Result<usize> {
        File::create(&self)?.write_json(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn test_str_to_json() {
        let s = r#"{"this": "is", "json": "parser"}"#;
        let ast_root = Value::parse(s);
        match ast_root {
            Ok(r) => assert_eq!(r["json"], Value::String("parser".to_string())),
            Err(_) => unreachable!("must be parsed as json"),
        }
    }

    #[test]
    fn test_string_to_json() {
        let s = r#"{"this": "is", "json": "parser"}"#.to_string();
        let ast_root = Value::parse(s);
        match ast_root {
            Ok(r) => assert_eq!(r["json"], Value::String("parser".to_string())),
            Err(_) => unreachable!("must be parsed as json"),
        }
    }

    #[test]
    fn test_file_io_json() {
        let json: RawJson = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "parser"],"#,
            r#"    "dict": {"one": 1, "two": 2, "three": 3}"#,
            r#"}"#,
        ]
        .into_iter()
        .collect();
        let result = || -> anyhow::Result<()> {
            let mut raw_json_file = tempfile::tempfile()?;
            write!(raw_json_file, "{json}")?;
            raw_json_file.seek(SeekFrom::Start(0))?;

            let ast_root1 = Value::parse_read(&raw_json_file)?;
            assert_eq!(ast_root1["language"], Value::String("rust".to_string()));
            let mut json_file1 = tempfile::tempfile()?;
            ast_root1.stringify_write(&json_file1, false)?;
            json_file1.seek(SeekFrom::Start(0))?;

            let ast_root2 = Value::parse_read(&json_file1)?;
            assert_eq!(ast_root2["language"], Value::String("rust".to_string()));
            let mut json_file2 = tempfile::tempfile()?;
            ast_root2.stringify_write(&json_file2, true)?;
            json_file2.seek(SeekFrom::Start(0))?;

            let ast_root3 = Value::parse_read(&json_file2)?;
            assert_eq!(ast_root3["language"], Value::String("rust".to_string()));

            assert_ne!(ast_root1.stringify(), json.to_string());
            assert_ne!(ast_root2.to_string(), json.to_string());
            assert_ne!(ast_root1.stringify(), ast_root2.to_string());

            assert_eq!(ast_root1, ast_root2);
            assert_eq!(ast_root2, ast_root3);
            assert_eq!(ast_root3, ast_root1);
            Ok(())
        }();
        assert!(result.is_ok());
    }
}
