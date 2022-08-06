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
            bail!("{}: unexpected token \"{c}\", while parse value", postr(pos))
        }
    }

    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        let mut object = HashMap::new();
        self.lexer.lex_1_char(Token::LeftBrace)?;
        while !self.lexer.is_next(Token::RightBrace) {
            if self.lexer.is_next(Token::Quotation) {
                let key = self.parse_string().context("while parse object's key")?;
                self.lexer.lex_1_char(Token::Colon).context("while parse object")?;
                let value = self.parse_value().context("while parse object's value")?;
                if let Ok((p, _comma)) = self.lexer.lex_1_char(Token::Comma) {
                    ensure!(!self.lexer.is_next(Token::RightBrace), "{}: trailing comma", postr(p));
                }
                object.insert(key.to_string(), value);
            }
        }
        self.lexer.lex_1_char(Token::RightBrace)?;
        Ok(Value::Object(object))
    }

    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        let mut array = Vec::new();
        self.lexer.lex_1_char(Token::LeftBracket)?;
        while !self.lexer.is_next(Token::RightBracket) {
            let value = self.parse_value()?;
            if let Ok((p, _comma)) = self.lexer.lex_1_char(Token::Comma) {
                ensure!(!self.lexer.is_next(Token::RightBracket), "{}: trailing comma", postr(p));
            }
            array.push(value);
        }
        self.lexer.lex_1_char(Token::RightBracket)?;
        Ok(Value::Array(array))
    }

    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        let &(pos, tf) =
            self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse bool"))?;
        if Token::tokenize(tf) == Token::Undecided('t') {
            let tru = self.lexer.lex_n_chars(4)?;
            ensure!("true" == tru, "{}: unexpected \"{tru}\", but expected \"true\"", postr(pos));
            Ok(Value::Bool(true))
        } else if Token::tokenize(tf) == Token::Undecided('f') {
            let fal = self.lexer.lex_n_chars(5)?;
            ensure!("false" == fal, "{}: unexpected \"{fal}\", but expected \"false\"", postr(pos));
            Ok(Value::Bool(false))
        } else {
            bail!("{}: no bool immediate start with '{tf}'", postr(pos));
        }
    }

    pub fn parse_null(&mut self) -> anyhow::Result<Value> {
        let &(pos, n) =
            self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse null"))?;
        if Token::tokenize(n) == Token::Undecided('n') {
            let null = self.lexer.lex_n_chars(4)?;
            ensure!("null" == null, "{}: unexpected \"{null}\", but expected \"null\"", postr(pos));
            Ok(Value::Null)
        } else {
            bail!("{}: no null immediate start with '{}'", postr(pos), n);
        }
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
        let empty = "{}".into();
        let mut parser = Parser::new(&empty);
        let object = parser.parse_object();
        if let Value::Object(m) = object.unwrap() {
            assert_eq!(m, HashMap::new());
        } else {
            unreachable!("\"{{}}\" must be parsed as empty object");
        }
        assert_eq!(parser.lexer.next(), None);
    }

    #[test]
    fn test_parse_empty_array() {
        let empty = "[\r\n \t \n  ]".into();
        let mut parser = Parser::new(&empty);
        let array = parser.parse_array();
        if let Value::Array(v) = array.unwrap() {
            assert_eq!(v, Vec::new());
        } else {
            unreachable!("\"[]\" must be parsed as empty array");
        }
        assert_eq!(parser.lexer.next(), None);
    }

    #[test]
    fn test_parse_bool() {
        let (itr, ifa) = ("true".into(), "false".into());
        let (mut tru_parser, mut fal_parser) = (Parser::new(&itr), Parser::new(&ifa));
        let (tru, fal) = (tru_parser.parse_bool(), fal_parser.parse_bool());
        if let (Value::Bool(t), Value::Bool(f)) = (tru.unwrap(), fal.unwrap()) {
            assert!(t && !f);
        } else {
            unreachable!("\"true\" and \"false\" must be parsed as bool immediate");
        }
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (None, None));
    }

    #[test]
    fn test_parse_null() {
        let inu = "null".into();
        let mut parser = Parser::new(&inu);
        let null = parser.parse_null();
        assert_eq!(Value::Null, null.unwrap());
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
