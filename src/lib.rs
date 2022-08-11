//! `dyson` is a dynamic json parser library.
//! use dyson, no need to define json scheme in advance.
//!
//! see [github](https://github.com/hayas1/dyson) also.
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
//! println!("{}", json["version"].evaluate_float()); // 0.1
//! println!("{:?}", &json["keyword"][Ranger(1..)]); // [Value::String("json"), Value::String("parser")]
//! println!("{:?}", json.get("foo")); // None
//!
//! // write json
//! json.dump("path/to/write.json").expect("failed to write json");
//! ```
//! more, see [`Value`] also.

pub mod ast;
pub mod rawjson;
pub mod syntax;

pub use ast::index::Ranger;
pub use ast::io::Indent;
pub use ast::Value;

fn postr((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}
