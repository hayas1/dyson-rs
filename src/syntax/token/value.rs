use crate::syntax::error::{ParseValueError, Pos, Positioned, TokenizeError};

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
        match c {
            '{' => Ok(Self::Object(ObjectToken::LeftBrace)),
            '[' => Ok(Self::Array(ArrayToken::LeftBracket)),
            't' | 'f' | 'n' => Ok(Self::Immediate(ImmediateToken::lookahead(c).expect("tokenize as immediate"))),
            '"' => Ok(Self::String(StringToken::Quotation)),
            // TODO allow leading plus(+)
            '-' | '0'..='9' => Ok(Self::Numeric(NumericToken::lookahead(c).expect("tokenize as numeric"))),
            // TODO allow comment
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(_s: &str) -> Result<Self, Self::Error> {
        // Self::lookahead(&LL1::ahead(s)?).map_err(|_| TokenizeError::UnmatchedToken { s: s.into() })
        unimplemented!()
    }
}
impl JsonToken for ValueToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseValueError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        match parser.lexer.branch() {
            Ok(Self::Object(_)) => Ok(ObjectToken::parse(parser).map_err(Pos::inherit)?),
            Ok(Self::Array(_)) => Ok(ArrayToken::parse(parser).map_err(Pos::inherit)?),
            Ok(Self::Immediate(_)) => Ok(ImmediateToken::parse(parser).map_err(Pos::inherit)?),
            Ok(Self::String(_)) => Ok(StringToken::parse(parser).map_err(Pos::inherit)?),
            Ok(Self::Numeric(_)) => Ok(NumericToken::parse(parser).map_err(Pos::inherit)?),
            Err(e) => Err(Pos::inherit(e))?,
        }
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

#[cfg(test)]
mod tests {
    use crate::{syntax::parser::Parser, Value};

    use super::*;

    #[test]
    fn test_str_object() {
        let object = r#"{"this": "is", "json": "parser"}"#.into();
        let mut parser = Parser::new(&object);
        let ast_root = ValueToken::parse(&mut parser).unwrap();
        assert_eq!(
            ast_root,
            Value::Object(
                vec![
                    ("this".to_string(), Value::String("is".to_string())),
                    ("json".to_string(), Value::String("parser".to_string()))
                ]
                .into_iter()
                .collect()
            )
        )
    }
}
