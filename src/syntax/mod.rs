pub(crate) mod error;
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod rawjson;
pub(crate) mod token;

pub type Position = (usize, usize);
