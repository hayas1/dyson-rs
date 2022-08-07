use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context as _};

use crate::{
    ast::Value,
    json::RawJson,
    lexer::Lexer,
    postr,
    token::{Context, Token},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(json: &'a RawJson) -> Self {
        Self { lexer: Lexer::new(json) }
    }

    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_whitespace();
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
                    let is_object_end = self.lexer.is_next(Token::RightBrace);
                    ensure!(!is_object_end, "{}: trailing comma", postr(p));
                } else {
                    // TODO no comma
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
                let is_array_end = self.lexer.is_next(Token::RightBracket);
                ensure!(!is_array_end, "{}: trailing comma", postr(p));
            } else {
                // TODO no comma
            }

            array.push(value);
        }
        self.lexer.lex_1_char(Token::RightBracket)?;
        Ok(Value::Array(array))
    }

    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        let &(pos, tf) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse bool"))?;
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
        let &(pos, n) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse null"))?;
        if Token::tokenize(n) == Token::Undecided('n') {
            let null = self.lexer.lex_n_chars(4)?;
            ensure!("null" == null, "{}: unexpected \"{null}\", but expected \"null\"", postr(pos));
            Ok(Value::Null)
        } else {
            bail!("{}: no null immediate start with '{}'", postr(pos), n);
        }
    }

    pub fn parse_string(&mut self) -> anyhow::Result<Value> {
        let mut string = String::new();
        let ((row, col), _quotation) = self.lexer.lex_1_char(Token::Quotation)?;
        while !self.lexer.is_next(Token::Quotation) {
            let &((r, _c), c) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, while parse string"))?;
            if row < r {
                bail!("{}: open string literal, must be closed by '\"'", postr((row, col)));
            } else if self.lexer.is_next(Token::ReverseSolidus) {
                string.push(self.parse_escape_sequence()?);
            } else {
                string.push(c);
                self.lexer.next();
            }
        }
        self.lexer.lex_1_char(Token::Quotation)?;
        Ok(Value::String(string))
    }

    pub fn parse_escape_sequence(&mut self) -> anyhow::Result<char> {
        let (p, _reverse_solidus) = self.lexer.lex_1_char(Token::ReverseSolidus)?;
        let (_, escape) = self.lexer.next().ok_or_else(|| anyhow!("unexpected EOF, while parse escape"))?;
        // FIXME better match case
        match Token::tokenize_with_context(escape, Some(Context::ParseString)) {
            Token::Quotation => Ok('"'),
            Token::ReverseSolidus => Ok('\\'),
            Token::Solidus => Ok('/'),
            Token::Backspace => {
                bail!("{}: unsupported {} escape sequence in Rust", Token::Backspace, postr(p))
            }
            Token::Formfeed => {
                bail!("{}: unsupported {} escape sequence in Rust", Token::Formfeed, postr(p))
            }
            Token::Linefeed => Ok('\n'),
            Token::CarriageReturn => Ok('\r'),
            Token::HorizontalTab => Ok('\t'),
            Token::Unicode => {
                let hex4digits = self.lexer.lex_n_chars(4)?;
                char::from_u32(u32::from_str_radix(&hex4digits[..], 16)?)
                    .ok_or_else(|| anyhow!("{}: cannot \\{hex4digits} convert to unicode", postr(p)))
            }
            Token::Unexpected(c) => bail!("{}: unexpected escape sequence{c}", postr(p)),
            _ => unreachable!("unexpected escape sequence is Token::Unexpected"),
        }
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
        let (tru, fal) = ("true".into(), "false".into());
        let (mut tru_parser, mut fal_parser) = (Parser::new(&tru), Parser::new(&fal));
        let (tru, fal) = (tru_parser.parse_bool().unwrap(), fal_parser.parse_bool().unwrap());
        if let (Value::Bool(t), Value::Bool(f)) = (tru, fal) {
            assert!(t && !f);
        } else {
            unreachable!("\"true\" and \"false\" must be parsed as bool immediate");
        }
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (None, None));

        let (tru3, f4lse) = ("tru3".into(), "f4lse".into());
        let (mut tru_parser, mut fal_parser) = (Parser::new(&tru3), Parser::new(&f4lse));
        let (tru3_err, f4lse_err) = (tru_parser.parse_bool().unwrap_err(), fal_parser.parse_bool().unwrap_err());
        assert!(tru3_err.to_string().contains("true"));
        assert!(tru3_err.to_string().contains("tru3"));
        assert!(f4lse_err.to_string().contains("false"));
        assert!(f4lse_err.to_string().contains("f4lse"));
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (None, None));
    }

    #[test]
    fn test_parse_null() {
        let null = "null".into();
        let mut parser = Parser::new(&null);
        let null = parser.parse_null().unwrap();
        assert_eq!(null, Value::Null);
        assert_eq!(parser.lexer.next(), None);

        let nuli = "nuli".into();
        let mut parser = Parser::new(&nuli);
        let nuli = parser.parse_null().unwrap_err();
        assert!(nuli.to_string().contains("null"));
        assert!(nuli.to_string().contains("nuli"));
        assert_eq!(parser.lexer.next(), None);
    }

    #[test]
    fn test_parse_string() {
        let string = r#""Rust""#.into();
        let mut parser = Parser::new(&string);
        let string = parser.parse_string().unwrap();
        assert_eq!(string, Value::String("Rust".to_string()));
        assert_eq!(parser.lexer.next(), None);

        let solidus = r#""Ru\/st""#.into();
        let mut parser = Parser::new(&solidus);
        let solidus = parser.parse_string().unwrap();
        assert_eq!(solidus, Value::String("Ru/st".to_string()));
        assert_eq!(parser.lexer.next(), None);

        let linefeed = r#""Ru\nst""#.into();
        let mut parser = Parser::new(&linefeed);
        let linefeed = parser.parse_string().unwrap();
        assert_eq!(linefeed, Value::String("Ru\nst".to_string()));
        assert_eq!(parser.lexer.next(), None);

        let unicode = r#""R\u00f9st""#.into();
        let mut parser = Parser::new(&unicode);
        let unicode = parser.parse_string().unwrap();
        assert_eq!(unicode, Value::String("Rùst".to_string()));
        assert_eq!(parser.lexer.next(), None);
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
