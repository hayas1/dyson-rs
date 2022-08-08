//! `dyson` is a dynamic json parser library.
//! use dyson, no need to define json scheme in advance.
//!
//! [see github](https://github.com/hayas1/dyson)

mod ast;
mod io;
mod json;
mod lexer;
mod parser;
mod token;

pub use crate::ast::*;
pub use crate::io::*;
pub use crate::json::*;
pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::token::*;

fn postr((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}
