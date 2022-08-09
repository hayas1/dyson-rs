//! `dyson` is a dynamic json parser library.
//! use dyson, no need to define json scheme in advance.
//!
//! [see github](https://github.com/hayas1/dyson)

pub mod ast;
pub mod json;
pub mod parser;

fn postr((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}
