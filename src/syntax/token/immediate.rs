use crate::syntax::error::{LexerError, ParseStringError, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol};

#[derive(PartialEq, Eq, Clone)]
pub enum ImmediateToken {
    True,
    False,
    Null,
}
impl std::fmt::Display for ImmediateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
        }
    }
}
impl std::fmt::Debug for ImmediateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            imm @ (Self::True | Self::False | Self::Null) => imm.fmt(f),
        }
    }
}
impl LL1Token for ImmediateToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        todo!()
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
impl JsonToken for ImmediateToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
