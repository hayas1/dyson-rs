use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context as _};

use crate::{ast::Value, json::RawJson, lexer::Lexer, postr, token::Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(json: &'a RawJson) -> Self {
        Self { lexer: Lexer::new(json) }
    }

    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, start parse value"))?;

        let tokenized = Token::tokenize(c);
        if matches!(tokenized, Token::LeftBrace) {
            self.parse_object()
        } else if matches!(tokenized, Token::LeftBracket) {
            self.parse_array()
        } else if matches!(tokenized, Token::Undecided('t') | Token::Undecided('f')) {
            self.parse_bool()
        } else if matches!(tokenized, Token::Undecided('n')) {
            self.parse_null()
        } else if matches!(tokenized, Token::Minus | Token::Digit) {
            self.parse_number()
        } else if matches!(tokenized, Token::Quotation) {
            self.parse_string()
        } else {
            bail!("{}: unexpected token \"{}\", while parse value", postr(pos), c)
        }
    }

    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(_pos, _lb) = peeked.ok_or_else(|| anyhow!("unexpected EOF, start parse object"))?;

        let mut object = HashMap::new();
        self.lexer.lex1char(Token::LeftBrace)?;
        while !self.lexer.is_next(Token::RightBrace) {
            let (pos, quotation) =
                self.lexer.next().ok_or_else(|| anyhow!("unexpected EOF, start parse object"))?;
            if matches!(Token::tokenize(quotation), Token::Quotation) {
                let key = self.parse_string().with_context(|| format!("{}: key", postr(pos)))?;
                self.lexer
                    .lex1char(Token::Colon)
                    .with_context(|| format!("{}: while parse object", postr(pos)))?;
                let value = self.parse_value().with_context(|| format!("{}: value", postr(pos)))?;
                if let Ok((p, _comma)) = self.lexer.lex1char(Token::Comma) {
                    ensure!(!self.lexer.is_next(Token::RightBrace), "{}: trailing comma", postr(p))
                }
                object.insert(key.to_string(), value);
            }
        }
        self.lexer.lex1char(Token::RightBrace)?;
        Ok(Value::Object(object))
    }

    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, start parse array"))?;
        Ok(Value::Array(Vec::new()))
    }

    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        // TODO
        Ok(Value::Bool(true))
    }

    pub fn parse_null(&mut self) -> anyhow::Result<Value> {
        // TODO
        Ok(Value::Null)
    }

    pub fn parse_string(&mut self) -> anyhow::Result<Value> {
        // TODO
        Ok(Value::String("".to_string()))
    }

    pub fn parse_number(&mut self) -> anyhow::Result<Value> {
        // TODO
        Ok(Value::Number("0".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_object() {
        let empty: RawJson = "{}".into();
        let mut parser = Parser::new(&empty);
        let object = parser.parse_object();
        if let Value::Object(om) = object.unwrap() {
            assert_eq!(om, HashMap::new());
        } else {
            unreachable!("\"{{}}\" must be parsed as empty object");
        }
    }

    #[test]
    fn test_parse_json() {
        let json: RawJson = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json""#,
            r#"    "version": 0.1"#,
            r#"    "keyword": ["rust", "json", "parser"]"#,
            r#"{"#,
        ]
        .into_iter()
        .collect();
    }
}
