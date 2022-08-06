mod ast;
mod json;
mod lexer;
mod parser;
mod token;

fn error_pos((row, col): (usize, usize)) -> String {
    format!("line {} (col {})", row, col)
}

pub use crate::ast::*;
pub use crate::json::*;
pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::token::*;
