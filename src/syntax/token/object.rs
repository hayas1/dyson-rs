use linked_hash_map::LinkedHashMap;

use crate::syntax::error::{ParseObjectError, Pos, Positioned, TokenizeError};

use super::{string::StringToken, value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ObjectToken {
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
}
impl std::fmt::Display for ObjectToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
        }
    }
}
impl LL1Token for ObjectToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            '{' => Ok(Self::LeftBrace),
            '}' => Ok(Self::RightBrace),
            ':' => Ok(Self::Colon),
            ',' => Ok(Self::Comma),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        Self::lookahead(&LL1::ahead(s)?).map_err(|_| TokenizeError::UnmatchedToken { s: s.into() })
    }
}
impl JsonToken for ObjectToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseObjectError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut object = LinkedHashMap::new();
        let (_, _left_brace) = parser.lexer.seek(Self::LeftBrace).map_err(Pos::inherit)?;
        while !matches!(parser.lexer.branch(), Ok(ObjectToken::RightBrace)) {
            if matches!(parser.lexer.branch(), Ok(StringToken::Quotation)) {
                let key = StringToken::parse(parser).map_err(Pos::inherit)?;
                let (_, _colon) = parser.lexer.seek(Self::Colon).map_err(Pos::inherit)?;
                let value = ValueToken::parse(parser).map_err(Pos::inherit)?;

                object.insert(key.into(), value);

                // TODO trailing comma
                if let Ok(((start, end), _comma)) = parser.lexer.seek(ObjectToken::Comma) {
                    if matches!(parser.lexer.branch(), Ok(ObjectToken::RightBrace)) {
                        return Err(Pos::with(ParseObjectError::TrailingComma {}, start, end))?;
                    }
                }
            } else {
                break;
            }
        }
        let (_, _right_brace) = parser.lexer.seek(ObjectToken::RightBrace).map_err(Pos::inherit)?;
        Ok(object.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{syntax::parser::Parser, Value};

    #[test]
    fn test_parse_empty_object() {
        let empty = "{}".into();
        let mut parser = Parser::new(&empty);
        if let Value::Object(map) = ObjectToken::parse(&mut parser).unwrap() {
            assert_eq!(map, LinkedHashMap::new());
        } else {
            unreachable!("\"{{}}\" must be parsed as empty object");
        }
        assert_eq!(parser.lexer.next(), Some(((0, 2), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }
}
