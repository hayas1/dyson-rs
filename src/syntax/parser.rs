use super::{
    error::{ParseError, ParseNumberError, ParseStringError, Position, StructureError},
    lexer::{Lexer, SkipWs},
    token::{EscapedStringToken, ImmediateToken, JsonToken, LL1Token, NumberToken},
};
use crate::ast::Value;
use anyhow::Context as _;
use linked_hash_map::LinkedHashMap;

pub struct Parser {}

impl Parser {
    /// get new parser to parse raw json
    pub fn new() -> Self {
        // TODO trailing comma, allow comment
        Self {}
    }

    /// parse `value` of json. the following ebnf is not precise.<br>
    /// `value` := `object` | `array` | `bool` | `null` | `string` | `number`;
    pub fn parse_value(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let expected = vec![JsonToken::LeftBrace, JsonToken::Immediate(ImmediateToken::True)];
        if let Some(&(pos, c)) = lexer.skip_whitespace() {
            match JsonToken::lookahead(c) {
                Ok(JsonToken::LeftBrace) => self.parse_object(lexer),
                Ok(JsonToken::LeftBracket) => self.parse_array(lexer),
                Ok(JsonToken::Immediate(_)) => self.parse_immediate(lexer),
                Ok(JsonToken::String(_)) => self.parse_string(lexer),
                Ok(JsonToken::Number(_)) => self.parse_number(lexer),
                Ok(found) => Err(ParseError::UnexpectedToken { expected, found, pos })?,
                Err(error) => Err(ParseError::TokenizeError::<JsonToken> { error, pos })?,
            }
        } else {
            let eof = lexer.json.eof();
            Err(ParseError::UnexpectedEof { expected, pos: eof })?
        }
    }

    /// parse `object` of json. the following ebnf is not precise.<br>
    /// `object` := "{" { `string` ":" `value` \[ "," \] }  "}"
    pub fn parse_object(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let mut object = LinkedHashMap::new();
        let (_, _left_brace) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::LeftBrace)?;
        while !lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace) {
            if lexer.is_next::<_, SkipWs<true>>(JsonToken::String(EscapedStringToken::Quotation)) {
                let key = self.parse_string(lexer)?;
                lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::Colon)?;
                let value = self.parse_value(lexer)?;
                object.insert(key.into(), value);

                if let Ok((p, _comma)) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::Comma) {
                    if lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace) {
                        return Err(StructureError::TrailingComma { pos: p })?;
                    }
                }
            } else {
                break;
            }
        }
        lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace)?;
        Ok(Value::Object(object))
    }

    /// parse `array` of json. the following ebnf is not precise.<br>
    /// `array` := "\[" { `value` \[ "," \] }  "\]"
    pub fn parse_array(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let mut array = Vec::new();
        let (_, _left_bracket) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::LeftBracket)?;
        while !lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket) {
            let value = self.parse_value(lexer)?;
            array.push(value);

            if let Ok((p, _comma)) = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::Comma) {
                if lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket) {
                    return Err(StructureError::TrailingComma { pos: p })?;
                }
            } else {
                break;
            }
        }
        lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBracket)?;
        Ok(Value::Array(array))
    }

    /// parse `immediate` of json. the following ebnf is not precise.<br>
    /// `immediate` := "true" | "false" | "null"
    pub fn parse_immediate(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let expected = || vec![ImmediateToken::True, ImmediateToken::False, ImmediateToken::Null];
        let &(pos, c) =
            lexer.peek().ok_or_else(|| ParseError::UnexpectedEof { expected: expected(), pos: lexer.json.eof() })?;
        match ImmediateToken::lookahead(c) {
            Ok(token @ (ImmediateToken::True | ImmediateToken::False | ImmediateToken::Null)) => {
                lexer.lex_expected(token.clone())?; // TODO error handling
                match token.value() {
                    Some(bl) => Ok(Value::Bool(bl)),
                    None => Ok(Value::Null),
                }
            }
            Err(error) => Err(ParseError::TokenizeError::<ImmediateToken> { error, pos })?,
        }
    }

    /// parse `string` of json. the following ebnf is not precise.<br>
    /// `string` := """ { `escape_sequence` | `char`  } """
    pub fn parse_string(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let mut string = String::new();
        let (start, _quotation) = lexer.lex_1_char::<_, SkipWs<false>>(EscapedStringToken::Quotation)?;
        while !lexer.is_next::<_, SkipWs<false>>(EscapedStringToken::Quotation) {
            let &(p, c) = lexer.peek().ok_or_else(|| {
                let eof = lexer.json.eof();
                ParseStringError::UnexpectedEof { comp: string.clone(), start, end: eof }
            })?;
            if c == '\n' {
                return Err(ParseStringError::UnexpectedLinefeed { comp: string, start, end: p })?;
            } else if lexer.is_next::<_, SkipWs<false>>(EscapedStringToken::ReverseSolidus) {
                string.push(self.parse_escape_sequence(lexer)?);
            } else {
                string.push(c);
                lexer.next();
            }
        }
        lexer.lex_1_char::<_, SkipWs<false>>(EscapedStringToken::Quotation)?;
        Ok(Value::String(string))
    }

    /// parse `escape_sequence` of json. the following ebnf is not precise.<br>
    /// `escape_sequence` := "\\"" | "\\\\" | "\\/" | "\n" | "\r" | "\t" | `unicode`
    pub fn parse_escape_sequence(&self, lexer: &mut Lexer) -> anyhow::Result<char> {
        let (start, reverse_solidus) = lexer.lex_1_char::<_, SkipWs<false>>(EscapedStringToken::ReverseSolidus)?;
        let (p, escaped) = lexer.next().ok_or_else(|| {
            let eof = lexer.json.eof();
            ParseStringError::UnexpectedEof { comp: reverse_solidus.to_string(), start, end: eof }
        })?;
        match EscapedStringToken::lookahead(escaped) {
            Ok(EscapedStringToken::Quotation) => Ok('"'),
            Ok(EscapedStringToken::ReverseSolidus) => Ok('\\'),
            Ok(EscapedStringToken::Solidus) => Ok('/'),
            Ok(escape @ (EscapedStringToken::Backspace | EscapedStringToken::Formfeed)) => {
                Err(ParseStringError::UnsupportedEscapeSequence { escape, start, end: p })?
            }
            Ok(EscapedStringToken::Linefeed) => Ok('\n'),
            Ok(EscapedStringToken::CarriageReturn) => Ok('\r'),
            Ok(EscapedStringToken::HorizontalTab) => Ok('\t'),
            Ok(EscapedStringToken::Unicode) => self.parse_unicode(lexer, start),
            Ok(escape @ EscapedStringToken::Hex4Digits(_)) => {
                Err(ParseStringError::UnexpectedEscapeSequence { escape, start, end: p })?
            }
            Err(error) => Err(ParseError::TokenizeError::<EscapedStringToken> { error, pos: start })?,
        }
    }

    /// parse `unicode` of json. the following ebnf is not precise.<br>
    /// `unicode` := "\u" `hex4digits`
    pub fn parse_unicode(&self, lexer: &mut Lexer, start: Position) -> anyhow::Result<char> {
        let (hex4, nexted) = lexer.lex_n_chars(4)?;
        let (p, _) = nexted.ok_or_else(|| {
            let eof = lexer.json.eof();
            ParseStringError::UnexpectedEof { comp: hex4.clone(), start, end: eof }
        })?;
        let uc = char::from_u32(u32::from_str_radix(&hex4, 16)?);
        Ok(uc.ok_or(ParseStringError::CannotConvertUnicode { uc: hex4, start, end: p })?)
    }

    /// parse `number` of json. the following ebnf is not precise.<br>
    /// `number` := \[ "-" \] `digits` \[ \[ `fraction_part` \] \[`exponent_part` \] \]
    pub fn parse_number(&self, lexer: &mut Lexer) -> anyhow::Result<Value> {
        let mut number = String::new();
        let &(start, _) = lexer.peek().ok_or_else(|| {
            let eof = lexer.json.eof();
            ParseNumberError::UnexpectedEof { num: number.clone(), start: eof, end: eof }
        })?;
        if let Ok((_c, minus)) = lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Minus) {
            number.push(minus);
        }
        if let Ok((_, zero)) = lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Zero) {
            number.push(zero);
        } else {
            number.push_str(&self.parse_digits(lexer, start)?);
        }

        let &(_, c) = lexer.peek().unwrap_or(&(lexer.json.eof(), '\0'));
        if matches!(NumberToken::lookahead(c), Ok(NumberToken::Dot | NumberToken::Exponent)) {
            if lexer.is_next::<_, SkipWs<false>>(NumberToken::Dot) {
                number.push_str(&self.parse_fraction(lexer, start)?);
            }
            if lexer.is_next::<_, SkipWs<false>>(NumberToken::Exponent) {
                number.push_str(&self.parse_exponent(lexer, start)?);
            }
            let &(end, _) = lexer.peek().unwrap_or(&(lexer.json.eof(), '\0'));
            Ok(Value::Float(number.parse().with_context(|| ParseNumberError::CannotConvertF64 {
                num: number,
                start,
                end,
            })?))
        } else {
            let eof = lexer.json.eof();
            let &(end, _) = lexer.peek().unwrap_or(&(eof, '\0'));
            Ok(Value::Integer(number.parse().with_context(|| ParseNumberError::CannotConvertI64 {
                num: number,
                start,
                end,
            })?))
        }
    }

    /// parse `digits` of json. the following ebnf is not precise.<br>
    /// `digits` := { "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" }
    fn parse_digits(&self, lexer: &mut Lexer, start: Position) -> anyhow::Result<String> {
        let mut digits = String::new();
        while let Some(&(_, c)) = lexer.peek() {
            if matches!(NumberToken::lookahead(c), Ok(NumberToken::Zero | NumberToken::OneNine(_))) {
                let (_, digit) = lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
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
    pub fn parse_fraction(&self, lexer: &mut Lexer, start: Position) -> anyhow::Result<String> {
        let mut fraction_component = String::new();
        let (_, dot) = lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Dot)?;
        fraction_component.push(dot);
        fraction_component.push_str(&self.parse_digits(lexer, start)?);
        Ok(fraction_component)
    }

    /// parse `exponent_part` of json. the following ebnf is not precise.<br>
    /// `exponent_part` := ("E" | "e") \[ "+" | "-" \] `digits`
    pub fn parse_exponent(&self, lexer: &mut Lexer, start: Position) -> anyhow::Result<String> {
        let mut exponent_component = String::new();
        let (_, exponent) = lexer.lex_1_char::<_, SkipWs<false>>(NumberToken::Exponent)?;
        exponent_component.push(exponent);
        let &(pos, sign_or_digits) = lexer.peek().ok_or_else(|| {
            let eof = lexer.json.eof();
            ParseNumberError::UnexpectedEof { num: exponent_component.clone(), start, end: eof }
        })?;
        match NumberToken::lookahead(sign_or_digits) {
            Ok(NumberToken::Plus | NumberToken::Minus) => {
                let (_, sign) = lexer.next().unwrap_or_else(|| unreachable!("previous peek ensure this next success"));
                exponent_component.push(sign)
            }
            Ok(NumberToken::Zero | NumberToken::OneNine(_)) => (),
            Ok(found @ (NumberToken::Dot | NumberToken::Exponent)) => {
                let mut expected = vec![NumberToken::Plus, NumberToken::Minus];
                expected.append(&mut ('0'..='9').map(|n| NumberToken::tokenize(&n.to_string()).unwrap()).collect());
                return Err(ParseNumberError::UnexpectedToken { expected, found, pos })?;
            }
            Err(error) => {
                Err(ParseError::TokenizeError::<NumberToken> { error, pos })?;
            }
        }
        exponent_component.push_str(&self.parse_digits(lexer, start)?);
        Ok(exponent_component)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_object() {
        let empty = "{}".into();
        let (mut lexer, parser) = (Lexer::new(&empty), Parser::new());
        let object = parser.parse_object(&mut lexer);
        if let Value::Object(m) = object.unwrap() {
            assert_eq!(m, LinkedHashMap::new());
        } else {
            unreachable!("\"{{}}\" must be parsed as empty object");
        }
        assert_eq!(lexer.next(), Some(((0, 2), '\n')));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_parse_empty_array() {
        let empty = "[\r\n \t \n  ]".into();
        let (mut lexer, parser) = (Lexer::new(&empty), Parser::new());
        let array = parser.parse_array(&mut lexer);
        if let Value::Array(v) = array.unwrap() {
            assert_eq!(v, Vec::new());
        } else {
            unreachable!("\"[]\" must be parsed as empty array");
        }
        assert_eq!(lexer.next(), Some(((2, 3), '\n')));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_parse_bool() {
        let parser = Parser::new();

        let (tru, fal) = ("true".into(), "false".into());
        let (mut true_lexer, mut false_lexer) = (Lexer::new(&tru), Lexer::new(&fal));
        let (true_value, false_value) =
            (parser.parse_immediate(&mut true_lexer).unwrap(), parser.parse_immediate(&mut false_lexer).unwrap());
        if let (Value::Bool(t), Value::Bool(f)) = (true_value, false_value) {
            assert!(t && !f);
        } else {
            unreachable!("\"true\" and \"false\" must be parsed as bool immediate");
        }
        assert_eq!((true_lexer.next(), false_lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        assert_eq!((true_lexer.next(), false_lexer.next()), (None, None));

        let (tru3, f4lse) = ("tru3".into(), "f4lse".into());
        let (mut tru3_lexer, mut f4lse_lexer) = (Lexer::new(&tru3), Lexer::new(&f4lse));
        let (tru3_err, f4lse_err) = (
            parser.parse_immediate(&mut tru3_lexer).unwrap_err(),
            parser.parse_immediate(&mut f4lse_lexer).unwrap_err(),
        );
        assert!(tru3_err.to_string().contains("true"));
        assert!(tru3_err.to_string().contains("tru3"));
        assert!(f4lse_err.to_string().contains("false"));
        assert!(f4lse_err.to_string().contains("f4lse"));
        assert_eq!((tru3_lexer.next(), f4lse_lexer.next()), (Some(((0, 4), '\n')), Some(((0, 5), '\n'))));
        assert_eq!((tru3_lexer.next(), f4lse_lexer.next()), (None, None));
    }

    #[test]
    fn test_parse_null() {
        let null = "null".into();
        let (mut lexer, parser) = (Lexer::new(&null), Parser::new());
        let null = parser.parse_immediate(&mut lexer).unwrap();
        assert_eq!(null, Value::Null);
        assert_eq!(lexer.next(), Some(((0, 4), '\n')));
        assert_eq!(lexer.next(), None);

        let nuli = "nuli".into();
        let (mut lexer, parser) = (Lexer::new(&nuli), Parser::new());
        let nuli = parser.parse_immediate(&mut lexer).unwrap_err();
        assert!(nuli.to_string().contains("null"));
        assert!(nuli.to_string().contains("nuli"));
        assert_eq!(lexer.next(), Some(((0, 4), '\n')));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_parse_string() {
        let string = r#""Rust""#.into();
        let (mut lexer, parser) = (Lexer::new(&string), Parser::new());
        let string = parser.parse_string(&mut lexer).unwrap();
        assert_eq!(string, Value::String("Rust".to_string()));
        assert_eq!(lexer.next(), Some(((0, 6), '\n')));
        assert_eq!(lexer.next(), None);

        let solidus = r#""Ru\"st""#.into();
        let (mut lexer, parser) = (Lexer::new(&solidus), Parser::new());
        let solidus = parser.parse_string(&mut lexer).unwrap();
        assert_eq!(solidus, Value::String("Ru\"st".to_string()));
        assert_eq!(lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(lexer.next(), None);

        let linefeed = r#""Ru\nst""#.into();
        let (mut lexer, parser) = (Lexer::new(&linefeed), Parser::new());
        let linefeed = parser.parse_string(&mut lexer).unwrap();
        assert_eq!(linefeed, Value::String("Ru\nst".to_string()));
        assert_eq!(lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(lexer.next(), None);

        let unicode = r#""R\u00f9st""#.into();
        let (mut lexer, parser) = (Lexer::new(&unicode), Parser::new());
        let unicode = parser.parse_string(&mut lexer).unwrap();
        assert_eq!(unicode, Value::String("Rùst".to_string()));
        assert_eq!(lexer.next(), Some(((0, 11), '\n')));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_parse_number() {
        let hundred = "100".into();
        let (mut lexer, parser) = (Lexer::new(&hundred), Parser::new());
        let hundred = parser.parse_number(&mut lexer).unwrap();
        assert_eq!(hundred, Value::Integer(100));
        assert_eq!(lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(lexer.next(), None);

        let half = "0.5".into();
        let (mut lexer, parser) = (Lexer::new(&half), Parser::new());
        let half = parser.parse_number(&mut lexer).unwrap();
        assert_eq!(half, Value::Float(0.5));
        assert_eq!(lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(lexer.next(), None);

        let thousand = "1E3".into();
        let (mut lexer, parser) = (Lexer::new(&thousand), Parser::new());
        let thousand = parser.parse_number(&mut lexer).unwrap();
        assert_eq!(thousand, Value::Float(1000.));
        assert_eq!(lexer.next(), Some(((0, 3), '\n')));
        assert_eq!(lexer.next(), None);

        let ten = "0.1e2".into();
        let (mut lexer, parser) = (Lexer::new(&ten), Parser::new());
        let ten = parser.parse_number(&mut lexer).unwrap();
        assert_eq!(ten, Value::Float(10.));
        assert_eq!(lexer.next(), Some(((0, 5), '\n')));
        assert_eq!(lexer.next(), None);
    }
}
