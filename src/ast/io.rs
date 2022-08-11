use super::Value;
use crate::{rawjson::RawJson, syntax::Parser};
use anyhow::Context as _;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};

impl Value {
    /// parse string like raw json into ast.
    /// # example
    /// ```
    /// use dyson::Value;
    /// let raw = r#"{ "key": [ 1, "two", 3, {"foo": {"bar": "baz"} } ] }"#;
    /// println!("{}", Value::parse(raw).unwrap());
    ///
    /// let raw2 = vec!["{", "\"key\": [", "1,", "\"two\",", "3,", "{\"foo\": {", "\"bar\": \"baz\"", "}", "}", "]", "}"];
    /// println!("{}", Value::parse(raw2.into_iter().collect::<String>()).unwrap());
    /// // or
    /// use dyson::rawjson::RawJson;
    /// println!("{}", Value::parse(raw2.into_iter().collect::<RawJson>()).unwrap());
    /// ```
    pub fn parse<J: Into<RawJson>>(j: J) -> anyhow::Result<Value> {
        let json = j.into();
        Parser::new(&json).parse_value()
    }
    /// parse file like raw json into ast. see [`Value::load`] also.
    /// # example
    /// ```
    /// use dyson::Value;
    /// use std::fs::File;
    /// let file = File::open("path/to/read.json").unwrap();
    /// let json = Value::read(file).unwrap();
    ///
    /// println!("{json}");
    /// ```
    pub fn read<R: Read>(r: R) -> anyhow::Result<Value> {
        let json: RawJson = BufReader::new(r).lines().map(|l| l.expect("could not read line")).collect();
        Value::parse(json)
    }
    /// parse raw json file specified by path into ast. see [`Value::parse`] also.
    /// # example
    /// ```
    /// use dyson::Value;
    /// // `path/to/read.json`
    /// // {
    /// //     "language": "rust",
    /// //     "notation": "json",
    /// //     "version": 0.1,
    /// //     "keyword": ["rust", "json", "parser"]
    /// // }
    /// let json = Value::load("path/to/read.json").unwrap();
    ///
    /// println!("{json}");
    /// // {"language":"rust","version":0.1,"keyword":["rust","json","parser"],"notation":"json"}
    /// ```
    pub fn load<P: AsRef<Path>>(p: P) -> anyhow::Result<Value> {
        let file = File::open(p)?;
        Self::read(file)
    }

    /// write ast to file. written string has proper indent. see [`Value::dump`] also.
    /// /// # example
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{ "key": [ 1, "two", 3, {"foo": {"bar": "baz"} } ] }"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// use std::fs::File;
    /// let file = File::create("path/to/write.json").unwrap();
    /// json.write(file).unwrap();
    /// ```
    pub fn write<W: Write>(&self, w: W) -> anyhow::Result<usize> {
        BufWriter::new(w).write(Indent::<1>::format(self).as_bytes()).context("could not write file")
    }
    /// write ast to file specified by path. written string has proper indent. see [`Value::stringify`] also.
    /// # example
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{ "key": [ 1, "two", 3, {"foo": {"bar": "baz"} } ] }"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// json.dump("path/to/write.json").unwrap();
    /// // or
    /// use std::path::Path;
    /// json.dump(Path::new("path/to/write.json")).unwrap();
    /// // or
    /// use std::path::PathBuf;
    /// json.dump(PathBuf::from("path").join("to").join("write.json")).unwrap();
    /// ```
    pub fn dump<P: AsRef<Path>>(&self, p: P) -> anyhow::Result<usize> {
        let file = File::create(p)?;
        self.write(file)
    }
    /// write ast to file with indent. see [`Value::write`] and [`Value::dump_with`] also.
    pub fn write_with<W: Write, F: JsonFormatter>(&self, w: W) -> anyhow::Result<usize> {
        BufWriter::new(w).write(F::format(self).as_bytes()).context("could not write file")
    }
    /// /// write ast to file specified by path with indent. see [`Indent`] also
    /// # example
    /// ```
    /// use dyson::{Indent, Value};
    /// let raw_json = r#"{ "key": [ 1, "two", 3, {"foo": {"bar": "baz"} } ] }"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// json.dump_with::<_, Indent<0>>("path/to/write.json");
    /// // {"key":[1,"two",3,{"foo":{"bar":"baz"}}]}
    ///
    /// json.dump_with::<_, Indent<1>>("path/to/write.json");
    /// // {
    /// //     "key": [
    /// //         1,
    /// //         "two",
    /// //         3,
    /// //         {
    /// //             "foo": {
    /// //                 "bar": "baz"
    /// //             }
    /// //         }
    /// //     ]
    /// // }
    ///
    /// // `Indent<2>` is not implement, so cause compile error
    /// // json.dump_with::<_, Indent<2>>("path/to/write.json");
    /// ```
    /// see `Value::to_string` and `Value::stringify` also.
    pub fn dump_with<P: AsRef<Path>, F: JsonFormatter>(&self, p: P) -> anyhow::Result<usize> {
        let file = File::create(p)?;
        self.write_with::<File, F>(file)
    }
}

/// dyson support 2 level indent output string.
/// - `Indent<0>`: no unnecessary space and linefeed is included. (minified)
///   - can be gotten by `Value::to_string`
/// - `Indent<1>`: normal json indent. (1 line, 1 element basically)
///   - can be gotten by `Value::stringify`
///
/// default is `Indent<1>`, so `Indent` mean `Indent<1>`.
/// see [`Value::write_with`] and [`Value::dump_with`] also.
pub struct Indent<const N: u8 = 1>();
pub trait JsonFormatter {
    fn format(value: &Value) -> String;
}
impl JsonFormatter for Indent<0> {
    fn format(value: &Value) -> String {
        value.to_string()
    }
}
impl JsonFormatter for Indent<1> {
    fn format(value: &Value) -> String {
        value.stringify()
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

            let ast_root1 = Value::read(&raw_json_file)?;
            assert_eq!(ast_root1["language"], Value::String("rust".to_string()));
            let mut json_file1 = tempfile::tempfile()?;
            ast_root1.write(&json_file1)?;
            json_file1.seek(SeekFrom::Start(0))?;

            let ast_root2 = Value::read(&json_file1)?;
            assert_eq!(ast_root2["language"], Value::String("rust".to_string()));
            let mut json_file2 = tempfile::tempfile()?;
            ast_root2.write_with::<_, Indent<0>>(&json_file2)?;
            json_file2.seek(SeekFrom::Start(0))?;

            let ast_root3 = Value::read(&json_file2)?;
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
