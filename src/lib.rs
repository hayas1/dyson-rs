mod ast;
mod json;
mod lexer;
mod parser;
mod token;

pub use crate::ast::*;
pub use crate::json::*;
pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::token::*;

fn postr((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row + 1, col + 1)
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
