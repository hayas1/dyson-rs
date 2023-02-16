use linked_hash_map::LinkedHashMap;

use crate::syntax::error::{ParseObjectError, ParserError, TokenizeError, WithPos};

use super::{string::StringToken, JsonToken, LL1Token, NonTerminalSymbol, LL1};

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
    type Error = ParserError<ParseObjectError<Self>>;
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut object = LinkedHashMap::new();
        let (_, _left_brace) = parser.lexer.seek(Self::LeftBrace)?;
        while !matches!(parser.lexer.decide(), Ok(ObjectToken::RightBrace)) {
            if matches!(parser.lexer.decide(), Ok(StringToken::Quotation)) {
                let key = StringToken::parse(parser)?;
            }
        }
        // let (_, _left_brace) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::LeftBrace)?;
        // while !lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace) {
        //     if lexer.is_next::<_, SkipWs<true>>(JsonToken::String(EscapedStringToken::Quotation)) {
        //         let key = self.parse_string(lexer)?;
        //         lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::Colon)?;
        //         let value = self.parse_value(lexer)?;
        //         object.insert(key.into(), value);

        //         if let Ok((p, _comma)) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::Comma) {
        //             if lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace) {
        //                 return Err(StructureError::TrailingComma { pos: p })?;
        //             }
        //         }
        //     } else {
        //         break;
        //     }
        // }
        // lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace)?;
        Ok(object.into())
    }
}
