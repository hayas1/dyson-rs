use super::{Lexer, MainToken, NumberToken, StringToken, Token};
use crate::{ast::Value, postr, rawjson::RawJson};
use anyhow::{anyhow, bail, ensure, Context as _};
use std::collections::HashMap;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// get new parser to parse raw json
    pub fn new(json: &'a RawJson) -> Self {
        Self { lexer: Lexer::new(json) }
    }

    /// parse `value` of json. the following ebnf is not precise.<br>
    /// `value` := `object` | `array` | `bool` | `null` | `string` | `number`;
    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let &(pos, c) = self.lexer.skip_whitespace().ok_or_else(|| anyhow!("unexpected EOF, start parse value"))?;

        let tokenized = MainToken::tokenize(c);
        if matches!(tokenized, MainToken::LeftBrace) {
            self.parse_object()
        } else if matches!(tokenized, MainToken::LeftBracket) {
            self.parse_array()
        } else if matches!(tokenized, MainToken::Undecided('t') | MainToken::Undecided('f')) {
            self.parse_bool()
        } else if matches!(tokenized, MainToken::Undecided('n')) {
            self.parse_null()
        } else if matches!(tokenized, MainToken::Quotation) {
            self.parse_string()
        } else if matches!(tokenized, MainToken::Minus | MainToken::Digit) {
            self.parse_number()
        } else {
            bail!("{}: unexpected token \"{c}\", while parse value", postr(pos))
        }
    }

    /// parse `object` of json. the following ebnf is not precise.<br>
    /// `object` := "{" { `string` ":" `value` \[ "," \] }  "}"
    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        let mut object = HashMap::new();
        let (pos, _left_brace) = self.lexer.lex_1_char(MainToken::LeftBrace, true)?;
        while !self.lexer.is_next(MainToken::RightBrace, true) {
            if self.lexer.is_next(MainToken::Quotation, true) {
                let key = self.parse_string().context("while parse object's key")?;
                self.lexer.lex_1_char(MainToken::Colon, true).context("while parse object")?;
                let value = self.parse_value().context("while parse object's value")?;

                // FIXME trailing comma and missing comma
                let is_object_end = self.lexer.is_next(MainToken::RightBrace, true);
                if let Ok((p, _comma)) = self.lexer.lex_1_char(MainToken::Comma, true) {
                    ensure!(!is_object_end, "{}: trailing comma", postr(p));
                } else {
                    ensure!(is_object_end, "{}: object should be end with '}}'", postr(pos));
                }

                object.insert(key.into(), value);
            }
        }
        self.lexer.lex_1_char(MainToken::RightBrace, true)?;
        Ok(Value::Object(object))
    }

    /// parse `array` of json. the following ebnf is not precise.<br>
    /// `array` := "\[" { `value` \[ "," \] }  "\]"
    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        let mut array = Vec::new();
        let (pos, _left_bracket) = self.lexer.lex_1_char(MainToken::LeftBracket, true)?;
        while !self.lexer.is_next(MainToken::RightBracket, true) {
            let value = self.parse_value()?;

            // FIXME trailing comma and missing comma
            let is_array_end = self.lexer.is_next(MainToken::RightBracket, true);
            if let Ok((p, _comma)) = self.lexer.lex_1_char(MainToken::Comma, true) {
                ensure!(!is_array_end, "{}: trailing comma", postr(p));
            } else {
                ensure!(is_array_end, "{}: array should be end with ']'", postr(pos));
            }

            array.push(value);
        }
        self.lexer.lex_1_char(MainToken::RightBracket, true)?;
        Ok(Value::Array(array))
    }

    /// parse `bool` of json. the following ebnf is not precise.<br>
    /// `bool` := "true" | "false"
    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        let &(pos, tf) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse bool"))?;
        match MainToken::tokenize(tf) {
            MainToken::Undecided('t') => {
                let tru = self.lexer.lex_n_chars(4)?;
                ensure!("true" == tru, "{}: unexpected \"{tru}\", but expected \"true\"", postr(pos));
                Ok(Value::Bool(true))
            }
            MainToken::Undecided('f') => {
                let fal = self.lexer.lex_n_chars(5)?;
                ensure!("false" == fal, "{}: unexpected \"{fal}\", but expected \"false\"", postr(pos));
                Ok(Value::Bool(false))
            }
            _ => bail!("{}: no bool immediate start with '{tf}'", postr(pos)),
        }
    }

    /// parse `null` of json. the following ebnf is not precise.<br>
    /// `null` := "null"
    pub fn parse_null(&mut self) -> anyhow::Result<Value> {
        let &(pos, n) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, start parse null"))?;
        match MainToken::tokenize(n) {
            MainToken::Undecided('n') => {
                let null = self.lexer.lex_n_chars(4)?;
                ensure!("null" == null, "{}: unexpected \"{null}\", but expected \"null\"", postr(pos));
                Ok(Value::Null)
            }
            _ => bail!("{}: no null immediate start with '{}'", postr(pos), n),
        }
    }

    /// parse `string` of json. the following ebnf is not precise.<br>
    /// `string` := """ { `escape_sequence` | `char`  } """
    pub fn parse_string(&mut self) -> anyhow::Result<Value> {
        let mut string = String::new();
        let ((row, col), _quotation) = self.lexer.lex_1_char(StringToken::Quotation, false)?;
        while !self.lexer.is_next(StringToken::Quotation, false) {
            let &((r, _c), c) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, while parse string"))?;
            if row < r {
                bail!("{}: open string literal, must be closed by '\"'", postr((row, col)));
            } else if self.lexer.is_next(StringToken::ReverseSolidus, false) {
                string.push(self.parse_escape_sequence()?);
            } else {
                string.push(c);
                self.lexer.next();
            }
        }
        self.lexer.lex_1_char(StringToken::Quotation, false)?;
        Ok(Value::String(string))
    }

    /// parse `escape_sequence` of json. the following ebnf is not precise.<br>
    /// `escape_sequence` := "\\"" | "\\\\" | "\\/" | "\n" | "\r" | "\t" | "\u" `hex4digits`
    pub fn parse_escape_sequence(&mut self) -> anyhow::Result<char> {
        let (p, _reverse_solidus) = self.lexer.lex_1_char(StringToken::ReverseSolidus, false)?;
        let (_, escape) = self.lexer.next().ok_or_else(|| anyhow!("unexpected EOF, while parse escape"))?;
        // FIXME better match case
        match StringToken::tokenize(escape) {
            StringToken::Quotation => Ok('"'),
            StringToken::ReverseSolidus => Ok('\\'),
            StringToken::Solidus => Ok('/'),
            StringToken::Backspace => bail!("{}: unsupported {} in Rust", StringToken::Backspace, postr(p)),
            StringToken::Formfeed => bail!("{}: unsupported {} in Rust", StringToken::Formfeed, postr(p)),
            StringToken::Linefeed => Ok('\n'),
            StringToken::CarriageReturn => Ok('\r'),
            StringToken::HorizontalTab => Ok('\t'),
            StringToken::Unicode => {
                let hex4digits = self.lexer.lex_n_chars(4)?;
                char::from_u32(u32::from_str_radix(&hex4digits, 16)?)
                    .ok_or_else(|| anyhow!("{}: cannot \\{hex4digits} convert to unicode", postr(p)))
            }
            StringToken::Unexpected(c) => bail!("{}: unexpected escape sequence{c}", postr(p)),
        }
    }

    /// parse `number` of json. the following ebnf is not precise.<br>
    /// `number` := \[ "-" \] `digits` \[ \[ `fraction_part` \] \[`exponent_part` \] \]
    pub fn parse_number(&mut self) -> anyhow::Result<Value> {
        let mut number = String::new();
        let &((row, col), _) = self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, while parse escape"))?;
        if let Ok(((r, _c), minus)) = self.lexer.lex_1_char(NumberToken::Minus, false) {
            ensure!(row >= r, "{}: unexpected linefeed, while parse number", postr((row, col)));
            number.push(minus);
        }
        if let Ok(((r, _c), zero)) = self.lexer.lex_1_char(NumberToken::Zero, false) {
            ensure!(row >= r, "{}: unexpected linefeed, while parse number", postr((row, col)));
            number.push(zero);
        } else {
            number.push_str(&self.parse_digits((row, col))?);
        }
        let &(_, c) = self.lexer.peek().unwrap_or(&((self.lexer.json.rows(), 0), '\0'));
        if matches!(NumberToken::tokenize(c), NumberToken::Dot | NumberToken::Exponent) {
            if self.lexer.is_next(NumberToken::Dot, false) {
                number.push_str(&self.parse_fraction((row, col))?);
            }
            if self.lexer.is_next(NumberToken::Exponent, false) {
                number.push_str(&self.parse_exponent((row, col))?);
            }
            Ok(Value::Float(
                number.parse().with_context(|| format!("{number} maybe valid number, but cannot convert to f64"))?,
            ))
        } else {
            Ok(Value::Integer(
                number.parse().with_context(|| format!("{number} maybe valid number, but cannot convert to i64"))?,
            ))
        }
    }

    /// parse `digits` of json. the following ebnf is not precise.<br>
    /// `digits` := { "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" }
    fn parse_digits(&mut self, (start_row, start_col): (usize, usize)) -> anyhow::Result<String> {
        let mut digits = String::new();
        while let Some(&((r, _c), c)) = self.lexer.peek() {
            if start_row >= r && matches!(NumberToken::tokenize(c), NumberToken::Zero | NumberToken::OneNine) {
                let (_, digit) =
                    self.lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
                digits.push(digit)
            } else {
                ensure!(!digits.is_empty(), "{}: empty digits is invalid in json", postr((start_row, start_col)));
                return Ok(digits);
            }
        }
        ensure!(!digits.is_empty(), "{}: empty digits is invalid in json", postr((start_row, start_col)));
        Ok(digits)
    }

    /// parse `fraction_part` of json. the following ebnf is not precise.<br>
    /// `fraction_part` := "." `digits`
    pub fn parse_fraction(&mut self, (start_row, start_col): (usize, usize)) -> anyhow::Result<String> {
        let mut fraction_component = String::new();
        let ((r, _c), dot) = self.lexer.lex_1_char(NumberToken::Dot, false)?;
        ensure!(start_row >= r, "{}: unexpected linefeed, while parse number", postr((start_row, start_col)));
        fraction_component.push(dot);
        fraction_component.push_str(&self.parse_digits((start_row, start_col))?);
        Ok(fraction_component)
    }

    /// parse `exponent_part` of json. the following ebnf is not precise.<br>
    /// `exponent_part` := ("E" | "e") \[ "+" | "-" \] `digits`
    pub fn parse_exponent(&mut self, (start_row, start_col): (usize, usize)) -> anyhow::Result<String> {
        let mut exponent_component = String::new();
        let ((r, _c), exponent) = self.lexer.lex_1_char(NumberToken::Exponent, false)?;
        ensure!(start_row >= r, "{}: unexpected linefeed, while parse number", postr((start_row, start_col)));
        exponent_component.push(exponent);
        let &((r, _c), sign_or_digits) =
            self.lexer.peek().ok_or_else(|| anyhow!("unexpected EOF, while parse exponent"))?;
        ensure!(start_row >= r, "{}: unexpected linefeed, while parse number", postr((start_row, start_col)));
        match NumberToken::tokenize(sign_or_digits) {
            NumberToken::Plus | NumberToken::Minus => {
                let (_, sign) =
                    self.lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
                exponent_component.push(sign)
            }
            NumberToken::Zero | NumberToken::OneNine => (),
            _ => bail!("{}: unexpected '{sign_or_digits}', but expected sign or digit", postr((start_row, start_col))),
        }
        exponent_component.push_str(&self.parse_digits((start_row, start_col))?);
        Ok(exponent_component)
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

        let solidus = r#""Ru\"st""#.into();
        let mut parser = Parser::new(&solidus);
        let solidus = parser.parse_string().unwrap();
        assert_eq!(solidus, Value::String("Ru\"st".to_string()));
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
    fn test_parse_number() {
        let hundred = "100".into();
        let mut parser = Parser::new(&hundred);
        let hundred = parser.parse_number().unwrap();
        assert_eq!(hundred, Value::Integer(100));
        assert_eq!(parser.lexer.next(), None);

        let half = "0.5".into();
        let mut parser = Parser::new(&half);
        let half = parser.parse_number().unwrap();
        assert_eq!(half, Value::Float(0.5));
        assert_eq!(parser.lexer.next(), None);

        let thousand = "1E3".into();
        let mut parser = Parser::new(&thousand);
        let thousand = parser.parse_number().unwrap();
        assert_eq!(thousand, Value::Float(1000.));
        assert_eq!(parser.lexer.next(), None);

        let ten = "0.1e2".into();
        let mut parser = Parser::new(&ten);
        let ten = parser.parse_number().unwrap();
        assert_eq!(ten, Value::Float(10.));
        assert_eq!(parser.lexer.next(), None);
    }
}
