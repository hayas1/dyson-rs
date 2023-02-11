use super::{
    error::{
        ParseNumberError, ParseStringError, ParseValueError, Position, SequentialTokenError, SingleTokenError,
        StructureError,
    },
    lexer::{Lexer, SkipWs},
    rawjson::RawJson,
    token::{ImmediateToken, MainToken, NumberToken, SingleToken, StringToken},
};
use crate::ast::Value;
use anyhow::Context as _;
use linked_hash_map::LinkedHashMap;

pub struct Parser<'a> {
    pub(crate) lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// get new parser to parse raw json
    pub fn new(json: &'a RawJson) -> Self {
        Self { lexer: Lexer::new(json) }
    }

    /// parse `value` of json. the following ebnf is not precise.<br>
    /// `value` := `object` | `array` | `bool` | `null` | `string` | `number`;
    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let examples = || vec![MainToken::LeftBrace, MainToken::Undecided('t'), MainToken::Digit('0')];
        if let Some(&(pos, c)) = self.lexer.skip_whitespace() {
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
            } else if matches!(tokenized, MainToken::Minus | MainToken::Digit(_)) {
                self.parse_number()
            } else {
                Err(ParseValueError::CannotStartParseValue { examples: examples(), found: tokenized, pos })?
            }
        } else {
            let eof = self.lexer.json.eof();
            Err(ParseValueError::UnexpectedEof { examples: examples(), pos: eof })?
        }
    }

    /// parse `object` of json. the following ebnf is not precise.<br>
    /// `object` := "{" { `string` ":" `value` \[ "," \] }  "}"
    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        let mut object = LinkedHashMap::new();
        let (_, _left_brace) = self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::LeftBrace)?;
        while !self.lexer.is_next::<_, SkipWs<true>>(MainToken::RightBrace) {
            if self.lexer.is_next::<_, SkipWs<true>>(MainToken::Quotation) {
                let key = self.parse_string()?;
                self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::Colon)?;
                let value = self.parse_value()?;
                object.insert(key.into(), value);

                if let Ok((p, _comma)) = self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::Comma) {
                    if self.lexer.is_next::<_, SkipWs<true>>(MainToken::RightBrace) {
                        return Err(StructureError::TrailingComma { pos: p })?;
                    }
                }
            } else {
                break;
            }
        }
        self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::RightBrace)?;
        Ok(Value::Object(object))
    }

    /// parse `array` of json. the following ebnf is not precise.<br>
    /// `array` := "\[" { `value` \[ "," \] }  "\]"
    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        let mut array = Vec::new();
        let (_, _left_bracket) = self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::LeftBracket)?;
        while !self.lexer.is_next::<_, SkipWs<true>>(MainToken::RightBracket) {
            let value = self.parse_value()?;
            array.push(value);

            if let Ok((p, _comma)) = self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::Comma) {
                if self.lexer.is_next::<_, SkipWs<true>>(MainToken::RightBracket) {
                    return Err(StructureError::TrailingComma { pos: p })?;
                }
            } else {
                break;
            }
        }
        self.lexer.lex_1_char::<_, SkipWs<true>>(MainToken::RightBracket)?;
        Ok(Value::Array(array))
    }

    /// parse `bool` of json. the following ebnf is not precise.<br>
    /// `bool` := "true" | "false"
    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        let expected = || vec![ImmediateToken::True, ImmediateToken::False];
        let &(pos, tf) = self.lexer.peek().ok_or_else(|| {
            let eof = self.lexer.json.eof();
            SequentialTokenError::UnexpectedEof { expected: expected(), start: eof, end: eof }
        })?;
        match ImmediateToken::tokenize(tf) {
            ImmediateToken::Undecided('t') => {
                self.lexer.lex_expected(ImmediateToken::True)?;
                Ok(Value::Bool(true))
            }
            ImmediateToken::Undecided('f') => {
                self.lexer.lex_expected(ImmediateToken::False)?;
                Ok(Value::Bool(false))
            }
            bl => Err(SingleTokenError::UnexpectedToken { expected: expected(), found: bl, pos })?,
        }
    }

    /// parse `null` of json. the following ebnf is not precise.<br>
    /// `null` := "null"
    pub fn parse_null(&mut self) -> anyhow::Result<Value> {
        let expected = || vec![ImmediateToken::Null];
        let &(pos, n) = self.lexer.peek().ok_or_else(|| {
            let eof = self.lexer.json.eof();
            SequentialTokenError::UnexpectedEof { expected: expected(), start: eof, end: eof }
        })?;
        match ImmediateToken::tokenize(n) {
            ImmediateToken::Undecided('n') => {
                self.lexer.lex_expected(ImmediateToken::Null)?;
                Ok(Value::Null)
            }
            nl => Err(SingleTokenError::UnexpectedToken { expected: expected(), found: nl, pos })?,
        }
    }

    /// parse `string` of json. the following ebnf is not precise.<br>
    /// `string` := """ { `escape_sequence` | `char`  } """
    pub fn parse_string(&mut self) -> anyhow::Result<Value> {
        let mut string = String::new();
        let (start, _quotation) = self.lexer.lex_1_char::<_, SkipWs<false>>(StringToken::Quotation)?;
        while !self.lexer.is_next::<_, SkipWs<false>>(StringToken::Quotation) {
            let &(p, c) = self.lexer.peek().ok_or_else(|| {
                let eof = self.lexer.json.eof();
                ParseStringError::UnexpectedEof { comp: string.clone(), start, end: eof }
            })?;
            if c == '\n' {
                return Err(ParseStringError::UnexpectedLinefeed { comp: string, start, end: p })?;
            } else if self.lexer.is_next::<_, SkipWs<false>>(StringToken::ReverseSolidus) {
                string.push(self.parse_escape_sequence()?);
            } else {
                string.push(c);
                self.lexer.next();
            }
        }
        self.lexer.lex_1_char::<_, SkipWs<false>>(StringToken::Quotation)?;
        Ok(Value::String(string))
    }

    /// parse `escape_sequence` of json. the following ebnf is not precise.<br>
    /// `escape_sequence` := "\\"" | "\\\\" | "\\/" | "\n" | "\r" | "\t" | `unicode`
    pub fn parse_escape_sequence(&mut self) -> anyhow::Result<char> {
        let (start, reverse_solidus) = self.lexer.lex_1_char::<_, SkipWs<false>>(StringToken::ReverseSolidus)?;
        let (p, escaped) = self.lexer.next().ok_or_else(|| {
            let eof = self.lexer.json.eof();
            ParseStringError::UnexpectedEof { comp: reverse_solidus.to_string(), start, end: eof }
        })?;
        let tokenized = StringToken::tokenize(escaped);
        match tokenized {
            StringToken::Quotation => Ok('"'),
            StringToken::ReverseSolidus => Ok('\\'),
            StringToken::Solidus => Ok('/'),
            StringToken::Backspace | StringToken::Formfeed => {
                Err(ParseStringError::UnsupportedEscapeSequence { escape: tokenized, start, end: p })?
            }
            StringToken::Linefeed => Ok('\n'),
            StringToken::CarriageReturn => Ok('\r'),
            StringToken::HorizontalTab => Ok('\t'),
            StringToken::Unicode => self.parse_unicode(start),
            _ => Err(ParseStringError::UnexpectedEscapeSequence { escape: tokenized, start, end: p })?,
        }
    }

    /// parse `unicode` of json. the following ebnf is not precise.<br>
    /// `unicode` := "\u" `hex4digits`
    pub fn parse_unicode(&mut self, start: Position) -> anyhow::Result<char> {
        let (hex4, nexted) = self.lexer.lex_n_chars(4)?;
        let (p, _) = nexted.ok_or_else(|| {
            let eof = self.lexer.json.eof();
            ParseStringError::UnexpectedEof { comp: hex4.clone(), start, end: eof }
        })?;
        let uc = char::from_u32(u32::from_str_radix(&hex4, 16)?);
        Ok(uc.ok_or(ParseStringError::CannotConvertUnicode { uc: hex4, start, end: p })?)
    }

    /// parse `number` of json. the following ebnf is not precise.<br>
    /// `number` := \[ "-" \] `digits` \[ \[ `fraction_part` \] \[`exponent_part` \] \]
    pub fn parse_number(&mut self) -> anyhow::Result<Value> {
        let mut number = String::new();
        let &(start, _) = self.lexer.peek().ok_or_else(|| {
            let eof = self.lexer.json.eof();
            ParseNumberError::UnexpectedEof { num: number.clone(), start: eof, end: eof }
        })?;
        if let Ok((_c, minus)) = self.lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Minus) {
            number.push(minus);
        }
        if let Ok((_, zero)) = self.lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Zero) {
            number.push(zero);
        } else {
            number.push_str(&self.parse_digits(start)?);
        }

        let &(_, c) = self.lexer.peek().unwrap_or(&(self.lexer.json.eof(), '\0'));
        if matches!(NumberToken::tokenize(c), NumberToken::Dot | NumberToken::Exponent) {
            if self.lexer.is_next::<_, SkipWs<false>>(NumberToken::Dot) {
                number.push_str(&self.parse_fraction(start)?);
            }
            if self.lexer.is_next::<_, SkipWs<false>>(NumberToken::Exponent) {
                number.push_str(&self.parse_exponent(start)?);
            }
            let &(end, _) = self.lexer.peek().unwrap_or(&(self.lexer.json.eof(), '\0'));
            Ok(Value::Float(number.parse().with_context(|| ParseNumberError::CannotConvertF64 {
                num: number,
                start,
                end,
            })?))
        } else {
            let eof = self.lexer.json.eof();
            let &(end, _) = self.lexer.peek().unwrap_or(&(eof, '\0'));
            Ok(Value::Integer(number.parse().with_context(|| ParseNumberError::CannotConvertI64 {
                num: number,
                start,
                end,
            })?))
        }
    }

    /// parse `digits` of json. the following ebnf is not precise.<br>
    /// `digits` := { "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" }
    fn parse_digits(&mut self, start: Position) -> anyhow::Result<String> {
        let mut digits = String::new();
        while let Some(&(_, c)) = self.lexer.peek() {
            if matches!(NumberToken::tokenize(c), NumberToken::Zero | NumberToken::OneNine(_)) {
                let (_, digit) =
                    self.lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
                digits.push(digit)
            } else if digits.is_empty() {
                return Err(ParseNumberError::EmptyDigits { pos: start })?;
            } else {
                return Ok(digits);
            }
        }
        if digits.is_empty() {
            Err(ParseNumberError::EmptyDigits { pos: start })?
        } else {
            Ok(digits)
        }
    }

    /// parse `fraction_part` of json. the following ebnf is not precise.<br>
    /// `fraction_part` := "." `digits`
    pub fn parse_fraction(&mut self, start: Position) -> anyhow::Result<String> {
        let mut fraction_component = String::new();
        let (_, dot) = self.lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Dot)?;
        fraction_component.push(dot);
        fraction_component.push_str(&self.parse_digits(start)?);
        Ok(fraction_component)
    }

    /// parse `exponent_part` of json. the following ebnf is not precise.<br>
    /// `exponent_part` := ("E" | "e") \[ "+" | "-" \] `digits`
    pub fn parse_exponent(&mut self, start: Position) -> anyhow::Result<String> {
        let mut exponent_component = String::new();
        let (_, exponent) = self.lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Exponent)?;
        exponent_component.push(exponent);
        let &(pos, sign_or_digits) = self.lexer.peek().ok_or_else(|| {
            let eof = self.lexer.json.eof();
            ParseNumberError::UnexpectedEof { num: exponent_component.clone(), start, end: eof }
        })?;
        match NumberToken::tokenize(sign_or_digits) {
            NumberToken::Plus | NumberToken::Minus => {
                let (_, sign) =
                    self.lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
                exponent_component.push(sign)
            }
            NumberToken::Zero | NumberToken::OneNine(_) => (),
            sd => {
                let mut expected = vec![NumberToken::Plus, NumberToken::Minus];
                expected.append(&mut ('0'..='9').map(NumberToken::tokenize).collect());
                return Err(SingleTokenError::UnexpectedToken { expected, found: sd, pos })?;
            }
        }
        exponent_component.push_str(&self.parse_digits(start)?);
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
            assert_eq!(m, LinkedHashMap::new());
        } else {
            unreachable!("\"{{}}\" must be parsed as empty object");
        }
        assert_eq!(parser.lexer.next(), Some(((0, 2), '\n')));
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
        assert_eq!(parser.lexer.next(), Some(((2, 3), '\n')));
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
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (None, None));

        let (tru3, f4lse) = ("tru3".into(), "f4lse".into());
        let (mut tru_parser, mut fal_parser) = (Parser::new(&tru3), Parser::new(&f4lse));
        let (tru3_err, f4lse_err) = (tru_parser.parse_bool().unwrap_err(), fal_parser.parse_bool().unwrap_err());
        assert!(tru3_err.to_string().contains("true"));
        assert!(tru3_err.to_string().contains("tru3"));
        assert!(f4lse_err.to_string().contains("false"));
        assert!(f4lse_err.to_string().contains("f4lse"));
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        assert_eq!((tru_parser.lexer.next(), fal_parser.lexer.next()), (None, None));
    }

    #[test]
    fn test_parse_null() {
        let null = "null".into();
        let mut parser = Parser::new(&null);
        let null = parser.parse_null().unwrap();
        assert_eq!(null, Value::Null);
        assert_eq!(parser.lexer.next(), Some(((0, 4), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let nuli = "nuli".into();
        let mut parser = Parser::new(&nuli);
        let nuli = parser.parse_null().unwrap_err();
        assert!(nuli.to_string().contains("null"));
        assert!(nuli.to_string().contains("nuli"));
        assert_eq!(parser.lexer.next(), Some(((0, 4), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }

    #[test]
    fn test_parse_string() {
        let string = r#""Rust""#.into();
        let mut parser = Parser::new(&string);
        let string = parser.parse_string().unwrap();
        assert_eq!(string, Value::String("Rust".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 6), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let solidus = r#""Ru\"st""#.into();
        let mut parser = Parser::new(&solidus);
        let solidus = parser.parse_string().unwrap();
        assert_eq!(solidus, Value::String("Ru\"st".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let linefeed = r#""Ru\nst""#.into();
        let mut parser = Parser::new(&linefeed);
        let linefeed = parser.parse_string().unwrap();
        assert_eq!(linefeed, Value::String("Ru\nst".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let unicode = r#""R\u00f9st""#.into();
        let mut parser = Parser::new(&unicode);
        let unicode = parser.parse_string().unwrap();
        assert_eq!(unicode, Value::String("Rùst".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 11), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }

    #[test]
    fn test_parse_number() {
        let hundred = "100".into();
        let mut parser = Parser::new(&hundred);
        let hundred = parser.parse_number().unwrap();
        assert_eq!(hundred, Value::Integer(100));
        assert_eq!(parser.lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let half = "0.5".into();
        let mut parser = Parser::new(&half);
        let half = parser.parse_number().unwrap();
        assert_eq!(half, Value::Float(0.5));
        assert_eq!(parser.lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let thousand = "1E3".into();
        let mut parser = Parser::new(&thousand);
        let thousand = parser.parse_number().unwrap();
        assert_eq!(thousand, Value::Float(1000.));
        assert_eq!(parser.lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let ten = "0.1e2".into();
        let mut parser = Parser::new(&ten);
        let ten = parser.parse_number().unwrap();
        assert_eq!(ten, Value::Float(10.));
        assert_eq!(parser.lexer.next(), Some(((0, 5), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }
}
