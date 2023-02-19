use crate::syntax::error::{ParseArrayError, Pos, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol, LL1};

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
        match c {
            '[' => Ok(Self::LeftBracket),
            ']' => Ok(Self::RightBracket),
            ',' => Ok(Self::Comma),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        Self::lookahead(&LL1::ahead(s)?).map_err(|_| TokenizeError::UnmatchedToken { s: s.into() })
    }
}
impl JsonToken for ArrayToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseArrayError<ValueToken>>;
    /// parse `array` of json. the following ebnf is not precise.<br>
    /// `array` := "\[" { `value` \[ "," \] }  "\]"
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut array = Vec::new();
        let ((_start, _), _left_bracket) = parser.lexer.seek(Self::LeftBracket).map_err(Pos::inherit)?;
        while !matches!(parser.lexer.branch(), Ok(ArrayToken::RightBracket)) {
            let value = ValueToken::parse(parser).map_err(Pos::inherit)?;

            array.push(value);

            // TODO trailing comma
            if let Ok(((start, end), _comma)) = parser.lexer.seek(ArrayToken::Comma) {
                if matches!(parser.lexer.branch(), Ok(ArrayToken::RightBracket)) {
                    return Err(Pos::with(ParseArrayError::TrailingComma {}, start, end))?;
                }
            } else {
                break;
            }
        }
        let (_, _right_bracket) = parser.lexer.seek(ArrayToken::RightBracket).map_err(Pos::inherit)?;
        Ok(array.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{syntax::parser::Parser, Value};

    #[test]
    fn test_tokenize() {
        assert!(matches!(ArrayToken::lookahead(&'{'), Err(_)));
        assert!(matches!(ArrayToken::lookahead(&'['), Ok(ArrayToken::LeftBracket)));
        assert!(matches!(ArrayToken::tokenize(","), Ok(ArrayToken::Comma)));
        assert!(matches!(ArrayToken::tokenize(";"), Err(_)));
    }

    #[test]
    fn test_parse_empty_object() {
        let empty = "[]".into();
        let mut parser = Parser::new(&empty);
        if let Value::Array(vec) = ArrayToken::parse(&mut parser).unwrap() {
            assert_eq!(vec, Vec::new());
        } else {
            unreachable!("\"[]\" must be parsed as empty array");
        }
        assert_eq!(parser.lexer.next(), Some(((0, 2), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }
}
