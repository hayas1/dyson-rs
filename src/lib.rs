//! `dyson` is a dynamic json parser library.
//! use dyson, no need to define json scheme in advance.
//!
//! see [github](https://github.com/hayas1/dyson) also.
//!
//! # usage
//! in `Cargo.toml`
//! ```toml
//! [dependencies]
//!     dyson = { git = "https://github.com/hayas1/dyson" }
//! ```
//!
//! # examples
//! ```no_run
//! // `path/to/read.json`
//! // {
//! //     "language": "rust",
//! //     "notation": "json",
//! //     "version": 0.1,
//! //     "keyword": ["rust", "json", "parser"]
//! // }
//! use dyson::{Value, Ranger};
//! // read json
//! let json = Value::load("path/to/read.json").expect("invalid path or json structure");
//!
//! // access json
//! if let Value::String(language) = &json["language"] {
//!     println!("{}", language) // rust
//! }
//! println!("{}", json["version"].float()); // 0.1
//! println!("{:?}", &json["keyword"][Ranger(1..)]); // [Value::String("json"), Value::String("parser")]
//! println!("{:?}", json.get("foo")); // None
//!
//! // edit json
//! let mut json = json;
//! json["language"] = "ruby".into();
//! println!("{}", json["language"].string()); // ruby
//! json["version"].swap(&mut 0.2.into());
//! println!("{}", json["version"].float()); // 0.2
//! json["keyword"].update_with(|v| v.iter().map(|k| Value::from(k.string().to_uppercase())).collect());
//! println!("{:?}", json["keyword"].array()); // ["RUST", "JSON", "PARSER"]
//!
//! // write json
//! json.dump("path/to/write.json").expect("failed to write json");
//! // {
//! //     "language": "ruby",
//! //     "notation": "json",
//! //     "version": 0.2,
//! //     "keyword": [
//! //         "RUST",
//! //         "JSON",
//! //         "PARSER"
//! //     ]
//! // }
//! ```
//! more, see [`Value`] also.

pub mod ast;
pub mod syntax;

pub use ast::index::Ranger;
pub use ast::io::Indent;
pub use ast::visit::DfsEvent;
pub use ast::Value;

pub use ast::diff::{diff_value, diff_value_detail};
