use crate::syntax::error::{ParseStringError, Pos, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, NonTerminalSymbol};

#[derive(Debug, PartialEq, Eq, Clone)]
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
impl LL1Token for ImmediateToken {
    type Error = TokenizeError<Self>;
    type Symbol = NonTerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            't' => Ok(Self::True),
            'f' => Ok(Self::False),
            'n' => Ok(Self::Null),
            &c => Err(TokenizeError::UnmatchedTokenPrefix { c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match s {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "null" => Ok(Self::Null),
            s => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for ImmediateToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let expected: Self = parser.lexer.branch().map_err(Pos::inherit)?;
        let _ = parser.lexer.seek(expected.clone()).map_err(Pos::inherit)?;
        match expected {
            ImmediateToken::True => Ok(true.into()),
            ImmediateToken::False => Ok(false.into()),
            ImmediateToken::Null => Ok(().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{syntax::parser::Parser, Value};

    #[test]
    fn test_parse_bool() {
        let (t, f) = ("true".into(), "false".into());
        let (mut tp, mut fp) = (Parser::new(&t), Parser::new(&f));
        let (tv, fv) = (ImmediateToken::parse(&mut tp).unwrap(), ImmediateToken::parse(&mut fp).unwrap());
        if let (Value::Bool(t), Value::Bool(f)) = (tv, fv) {
            assert!(t && !f);
        } else {
            unreachable!("\"true\" and \"false\" must be parsed as bool immediate");
        }
        assert_eq!((tp.lexer.next(), fp.lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        assert_eq!((tp.lexer.next(), fp.lexer.next()), (None, None));

        // let (tru3, f4lse) = ("tru3".into(), "f4lse".into());
        // let (mut tp, mut fp) = (Parser::new(&tru3), Parser::new(&f4lse));
        // let (te, fe) = (ImmediateToken::parse(&mut tp).unwrap_err(), ImmediateToken::parse(&mut fp).unwrap_err());
        // // assert!(te.to_string().contains("true"));
        // // assert!(te.to_string().contains("tru3"));
        // // assert!(fe.to_string().contains("false"));
        // // assert!(fe.to_string().contains("f4lse"));
        // assert_eq!((tp.lexer.next(), fp.lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        // assert_eq!((tp.lexer.next(), fp.lexer.next()), (None, None));
    }
}
