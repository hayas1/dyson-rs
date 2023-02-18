use crate::syntax::error::{LexerError, ParseStringError, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayToken {
    LeftBracket,
    RightBracket,
    Comma,
}

impl std::fmt::Display for ArrayToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Comma => write!(f, ","),
        }
    }
}
impl LL1Token for ArrayToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        todo!()
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
impl JsonToken for ArrayToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
