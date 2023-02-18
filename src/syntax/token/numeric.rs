use crate::syntax::error::{LexerError, ParseStringError, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol, TerminalSymbol, LL1};

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
impl LL1Token for NumericToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        todo!()
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
impl JsonToken for NumericToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
