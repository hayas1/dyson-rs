use super::Value;
use crate::{rawjson::RawJson, syntax::Parser};
use anyhow::Context as _;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};

impl Value {
    /// parse raw json into ast.
    pub fn parse<J: Into<RawJson>>(j: J) -> anyhow::Result<Value> {
        let json = j.into();
        Parser::new(&json).parse_value()
    }
    /// parse file like raw json into ast. see [parse](Value) also.
    pub fn parse_read<R: Read>(r: R) -> anyhow::Result<Value> {
        let json = BufReader::new(r).lines().map(|l| l.expect("could not read line")).collect();
        Parser::new(&json).parse_value()
    }
    /// parse raw json file specified by path into ast. see [parse](Value) also.
    pub fn parse_path<P: AsRef<Path>>(p: P) -> anyhow::Result<Value> {
        let file = File::open(p)?;
        Self::parse_read(file)
    }
    /// write ast to file. written string has proper indent. see [stringify](Value) also.
    pub fn stringify_write<W: Write>(&self, w: W) -> anyhow::Result<usize> {
        BufWriter::new(w).write(self.stringify().as_bytes()).context("could not write file")
    }
    /// write ast to file specified by path. written string has proper indent. see [stringify](Value) also.
    pub fn stringify_path<P: AsRef<Path>>(&self, p: P) -> anyhow::Result<usize> {
        let file = File::create(p)?;
        self.stringify_write(file)
    }
    /// write ast to file. if `level` is `0`, no unnecessary space and linefeed is included.
    /// see [to_string](Value) also.
    pub fn stringify_write_with<W: Write>(&self, w: W, level: u8) -> anyhow::Result<usize> {
        let write = match level {
            0 => self.to_string(),
            _ => self.stringify(),
        };
        BufWriter::new(w).write(write.as_bytes()).context("could not write file")
    }
    /// write ast to file specified by path. if `level` is `0`, no unnecessary space and linefeed is included.
    /// see [to_string](Value) also.
    pub fn stringify_path_with<P: AsRef<Path>>(&self, p: P, level: u8) -> anyhow::Result<usize> {
        let file = File::create(p)?;
        self.stringify_write_with(file, level)
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
            ast_root1.stringify_write(&json_file1)?;
            json_file1.seek(SeekFrom::Start(0))?;

            let ast_root2 = Value::parse_read(&json_file1)?;
            assert_eq!(ast_root2["language"], Value::String("rust".to_string()));
            let mut json_file2 = tempfile::tempfile()?;
            ast_root2.stringify_write_with(&json_file2, 0)?;
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
