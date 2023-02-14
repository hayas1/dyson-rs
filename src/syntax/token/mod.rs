pub mod array;
pub mod immediate;
pub mod numeric;
pub mod object;
pub mod string;
pub mod value;

use super::parser::Parser;
use crate::ast::Value;

// TODO better implementation of Debug
// TODO implement parase
pub trait LL1Token: PartialEq + std::fmt::Display + std::fmt::Debug + Send + Sync + Sized {
    type Error: std::fmt::Display + std::fmt::Debug + Send + Sync;
    fn parse(parser: &mut Parser) -> Result<Value, Self::Error>;
    fn lookahead(c: &char) -> Option<Self>;
}
