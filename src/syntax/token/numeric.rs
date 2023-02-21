use std::fmt::Debug;

use crate::syntax::error::{LexerError, ParseNumericError, Pos, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, TerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumericToken {
    Zero,
    OneNine(LL1),
    Plus,
    Minus,
    Fraction(FractionToken),
    Exponent(ExponentToken), // TODO distinguish `E` from `e` ?
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DigitsToken {
    Digit(LL1),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FractionToken {
    Dot,
    Digit(LL1),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExponentToken {
    Digit(LL1),
    Plus,
    Minus,
    Exponent, // TODO distinguish `E` from `e` ?
}

impl std::fmt::Display for NumericToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::OneNine(c) => write!(f, "{}", c),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Fraction(t) => write!(f, "{}", t),
            Self::Exponent(t) => write!(f, "{}", t),
        }
    }
}
impl LL1Token for NumericToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Zero),
            &c @ '1'..='9' => Ok(Self::OneNine(LL1::Lookahead(c))),
            '+' => Ok(Self::Plus),
            '-' => Ok(Self::Minus),
            '.' => Ok(Self::Fraction(FractionToken::lookahead(c)?)),
            'e' | 'E' => Ok(Self::Exponent(ExponentToken::lookahead(c)?)),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match Self::lookahead(&LL1::ahead(s)?) {
            Ok(Self::OneNine(LL1::Lookahead(c))) => Ok(Self::OneNine(LL1::Tokenized(c.to_string()))),
            Ok(t) => Ok(t),
            Err(_) => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for NumericToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseNumericError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut numeric = String::new();
        let start = parser.lexer.pos();
        if let Ok(minus @ NumericToken::Minus) = parser.lexer.branch() {
            numeric.push_str(&minus.to_string());
        }
        match parser.lexer.branch() {
            Ok(zero @ NumericToken::Zero) => {
                numeric.push_str(&parser.lexer.seek(zero).expect("previous lookahead ensure this seek success").1)
            }
            Ok(NumericToken::OneNine(_)) => numeric.push_str(&DigitsToken::parse(parser).map_err(Pos::inherit)?),
            Ok(_) => (),
            Err(e) => Err(Pos(e))?,
        }
        if let Ok(NumericToken::Fraction(_) | NumericToken::Exponent(_)) = parser.lexer.branch() {
            if let Ok(NumericToken::Fraction(_)) = parser.lexer.branch() {
                numeric.push_str(&FractionToken::parse(parser)?);
            }
            if let Ok(NumericToken::Exponent(_)) = parser.lexer.branch() {
                numeric.push_str(&ExponentToken::parse(parser)?);
            }
            let float: f64 = numeric.parse().expect("parsed string must be convert to integer");
            Ok(float.into())
        } else {
            let integer: i64 = numeric.parse().expect("parsed string must be convert to integer");
            Ok(integer.into())
        }
    }
}

impl std::fmt::Display for DigitsToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Digit(c) => write!(f, "{}", c),
        }
    }
}
impl LL1Token for DigitsToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            &c @ '0'..='9' => Ok(Self::Digit(LL1::Lookahead(c))),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match Self::lookahead(&LL1::ahead(s)?) {
            Ok(Self::Digit(LL1::Lookahead(c))) => Ok(Self::Digit(LL1::Tokenized(c.to_string()))),
            Ok(t) => Ok(t),
            Err(_) => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for DigitsToken {
    type Output = String;
    type Error = Positioned<ParseNumericError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut digits = String::new();
        while let Ok(DigitsToken::Digit(LL1::Lookahead(c))) = parser.lexer.branch() {
            let expected = DigitsToken::Digit(LL1::Tokenized(c.to_string()));
            digits.push_str(&parser.lexer.seek(expected).expect("previous lookahead ensure this seek success").1);
        }
        Ok(digits)
    }
}

impl std::fmt::Display for FractionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Digit(c) => write!(f, "{}", c),
            Self::Dot => write!(f, "."),
        }
    }
}
impl LL1Token for FractionToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            &c @ '0'..='9' => Ok(Self::Digit(LL1::Lookahead(c))),
            '.' => Ok(Self::Dot),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match Self::lookahead(&LL1::ahead(s)?) {
            Ok(Self::Digit(LL1::Lookahead(c))) => Ok(Self::Digit(LL1::Tokenized(c.to_string()))),
            Ok(t) => Ok(t),
            Err(_) => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for FractionToken {
    type Output = String;
    type Error = Positioned<ParseNumericError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
impl From<FractionToken> for NumericToken {
    fn from(token: FractionToken) -> Self {
        Self::Fraction(token)
    }
}
impl From<LexerError<FractionToken>> for LexerError<NumericToken> {
    fn from(value: LexerError<FractionToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}
impl From<TokenizeError<FractionToken>> for TokenizeError<NumericToken> {
    fn from(value: TokenizeError<FractionToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}

impl std::fmt::Display for ExponentToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Digit(c) => write!(f, "{}", c),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Exponent => write!(f, "e"), // TODO E ?
        }
    }
}
impl LL1Token for ExponentToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            &c @ '0'..='9' => Ok(Self::Digit(LL1::Lookahead(c))),
            '+' => Ok(Self::Plus),
            '-' => Ok(Self::Minus),
            'e' | 'E' => Ok(Self::Exponent),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match Self::lookahead(&LL1::ahead(s)?) {
            Ok(Self::Digit(LL1::Lookahead(c))) => Ok(Self::Digit(LL1::Tokenized(c.to_string()))),
            Ok(t) => Ok(t),
            Err(_) => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for ExponentToken {
    type Output = String;
    type Error = Positioned<ParseNumericError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        todo!()
    }
}
impl From<ExponentToken> for NumericToken {
    fn from(token: ExponentToken) -> Self {
        Self::Exponent(token)
    }
}
impl From<LexerError<ExponentToken>> for LexerError<NumericToken> {
    fn from(value: LexerError<ExponentToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}
impl From<TokenizeError<ExponentToken>> for TokenizeError<NumericToken> {
    fn from(value: TokenizeError<ExponentToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{syntax::parser::Parser, Value};

    #[test]
    fn test_tokenize() {
        assert!(matches!(NumericToken::lookahead(&'0'), Ok(NumericToken::Zero)));
        assert!(matches!(NumericToken::lookahead(&'7'), Ok(NumericToken::OneNine(LL1::Lookahead('7')))));
    }

    #[test]
    fn test_parse_string() {
        let integer = r#"123456"#.into();
        let mut parser = Parser::new(&integer);
        let string = NumericToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::Integer(123456));
        assert_eq!(parser.lexer.next(), Some(((0, 6), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }
}
