use super::error::{ConvertError, TokenizeError};

pub mod array;
pub mod immediate;
pub mod numeric;
pub mod object;
pub mod string;
pub mod value;

pub trait JsonToken: LL1Token {
    type Output;
    type Error: std::fmt::Debug + Send + Sync;
    fn parse(parser: &mut super::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error>;
}
pub trait LL1Token: PartialEq + std::fmt::Display + std::fmt::Debug + Send + Sync + Sized {
    type Error: std::fmt::Debug + Send + Sync;
    type Symbol: SkipWhiteSpace;
    fn lookahead(c: &char) -> Result<Self, Self::Error>;
    fn tokenize(s: &str) -> Result<Self, Self::Error>;
}

#[derive(PartialEq, Eq, Clone)]
pub enum LL1 {
    Lookahead(char),
    Tokenized(String),
}
impl std::fmt::Display for LL1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lookahead(c) => c.fmt(f),
            Self::Tokenized(s) => s.fmt(f),
        }
    }
}
impl std::fmt::Debug for LL1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lookahead(c) => c.fmt(f),
            Self::Tokenized(s) => s.fmt(f),
        }
    }
}
impl LL1 {
    pub fn ahead(s: &str) -> Result<char, ConvertError> {
        let mut chars = s.chars();
        let first = chars.next();
        if chars.next().is_none() {
            first.ok_or_else(|| ConvertError::EmptyString)
        } else {
            Err(ConvertError::TooLongString { s: s.into() })
        }
    }
}

pub trait SkipWhiteSpace {
    fn skip_ws() -> bool;
    fn whitespace(c: &char) -> bool {
        matches!(c, ' ' | '\n' | '\r' | '\t')
    }
}
pub enum TerminalSymbol {}
impl SkipWhiteSpace for TerminalSymbol {
    fn skip_ws() -> bool {
        false
    }
}
pub enum NonTerminalSymbol {}
impl SkipWhiteSpace for NonTerminalSymbol {
    fn skip_ws() -> bool {
        true
    }
}
