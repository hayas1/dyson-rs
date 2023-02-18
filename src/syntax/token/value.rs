use crate::syntax::error::{LexerError, ParseObjectError, ParseStringError, Positioned, TokenizeError};

use super::{
    array::ArrayToken, immediate::ImmediateToken, numeric::NumericToken, object::ObjectToken, string::StringToken,
    JsonToken, LL1Token, NonTerminalSymbol,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueToken {
    Object(ObjectToken),
    Array(ArrayToken),
    Immediate(ImmediateToken),
    String(StringToken),
    Numeric(NumericToken),
}
impl<'a> std::fmt::Display for ValueToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(t) => t.fmt(f),
            Self::Array(t) => t.fmt(f),
            Self::Immediate(t) => t.fmt(f),
            Self::String(t) => t.fmt(f),
            Self::Numeric(t) => t.fmt(f),
        }
    }
}
impl LL1Token for ValueToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        todo!()
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
impl JsonToken for ValueToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseObjectError<ValueToken>>; // TODO ParseValueError
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}

impl From<ObjectToken> for ValueToken {
    fn from(token: ObjectToken) -> Self {
        Self::Object(token)
    }
}
impl From<ArrayToken> for ValueToken {
    fn from(token: ArrayToken) -> Self {
        Self::Array(token)
    }
}
impl From<ImmediateToken> for ValueToken {
    fn from(token: ImmediateToken) -> Self {
        Self::Immediate(token)
    }
}
impl From<StringToken> for ValueToken {
    fn from(token: StringToken) -> Self {
        Self::String(token)
    }
}
impl From<NumericToken> for ValueToken {
    fn from(token: NumericToken) -> Self {
        Self::Numeric(token)
    }
}
