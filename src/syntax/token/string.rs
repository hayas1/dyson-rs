use crate::syntax::error::{ParseObjectError, ParseStringError, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, TerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringToken {
    Quotation,
    ReverseSolidus,
    Solidus,
    Backspace,
    Formfeed,
    Linefeed,
    CarriageReturn,
    HorizontalTab,
    Unicode,
    Hex4Digits(LL1),
    Unescaped(LL1),
}
impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "\""),
            Self::ReverseSolidus => write!(f, "\\"),
            Self::Solidus => write!(f, "/"),
            Self::Backspace => write!(f, "\\b"),
            Self::Formfeed => write!(f, "\\f"),
            Self::Linefeed => write!(f, "\\n"),
            Self::CarriageReturn => write!(f, "\\r"),
            Self::HorizontalTab => write!(f, "\\t"),
            Self::Unicode => write!(f, "\\u"),
            Self::Hex4Digits(c) => write!(f, "{}", c),
            Self::Unescaped(c) => write!(f, "{}", c),
        }
    }
}

impl LL1Token for StringToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        todo!()
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
impl JsonToken for StringToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
