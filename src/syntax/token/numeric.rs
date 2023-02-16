use crate::syntax::error::ParserError;

use super::{LL1Token, TerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumericToken {
    Zero,
    OneNine(LL1),
    Plus,
    Minus,
    Dot,
    Exponent, // TODO distinguish `E` from `e` ?
}
impl std::fmt::Display for NumericToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::OneNine(c) => write!(f, "{}", c),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Dot => write!(f, "."),
            Self::Exponent => write!(f, "E"),
        }
    }
}
// impl LL1Token for NumericToken {
//     type Error = ErrorWithPosition<anyhow::Error>;
//     type Symbol = TerminalSymbol;
//     fn lookahead(c: &char) -> Option<Self> {
//         todo!()
//     }
//     fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<crate::ast::Value, Self::Error> {
//         todo!()
//     }
// }
