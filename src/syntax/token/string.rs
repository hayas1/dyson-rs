use crate::syntax::error::{LexerError, ParseStringError, Pos, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, TerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringToken {
    Quotation,
    ReverseSolidus,
    Unescaped(LL1),
    Escaped(EscapedStringToken),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EscapedStringToken {
    Quotation,
    ReverseSolidus,
    Solidus,
    Backspace,
    Formfeed,
    Linefeed,
    CarriageReturn,
    HorizontalTab,
    Unicode(EscapedUnicodeToken),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EscapedUnicodeToken {
    Hex4Digits(LL1),
}

impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "\""),
            Self::ReverseSolidus => write!(f, "\\"),
            Self::Escaped(e) => write!(f, "\\{}", e),
            Self::Unescaped(u) => u.fmt(f),
        }
    }
}

impl LL1Token for StringToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            '"' => Ok(Self::Quotation),
            '\\' => Ok(Self::ReverseSolidus),
            &c => Ok(Self::Unescaped(LL1::Lookahead(c))),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        let v = s.chars().collect::<Vec<_>>();
        if let Some(c) = v.get(0) {
            match Self::lookahead(c) {
                Ok(t) if v.len() == 1 => return Ok(t), // TODO \u ?
                Ok(Self::ReverseSolidus) if v.len() == 2 => return Self::lookahead(&v[1]), // TODO unexpected escape
                Ok(Self::ReverseSolidus) if v.len() == 5 => todo!("implement tokenize hex4digits"),
                Ok(_) | Err(_) => todo!(),
            }
        }
        Ok(Self::Unescaped(LL1::Tokenized(s.to_string())))
    }
}
impl JsonToken for StringToken {
    type Output = crate::ast::Value;
    type Error = Positioned<ParseStringError<ValueToken>>;
    /// parse `string` of json. the following ebnf is not precise.<br>
    /// `string` := """ { `escape_sequence` | `char`  } """
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut string = String::new();
        let ((start, _), _quotation) = parser.lexer.seek(StringToken::Quotation).map_err(Pos::inherit)?;
        while !matches!(parser.lexer.branch(), Ok(StringToken::Quotation)) {
            match parser.lexer.branch() {
                Ok(StringToken::Unescaped(LL1::Lookahead('\n'))) => {
                    let pos = parser.lexer.pos();
                    Err(Pos::with(ParseStringError::CannotClose { building: string.clone() }, start, pos))?
                }
                Ok(StringToken::ReverseSolidus) => string.push(EscapedStringToken::parse(parser)?),
                Ok(_unescaped) => string.push(parser.lexer.next().expect("previous peek ensure this next success").1),
                Err(e) => Err(Pos::inherit(e))?,
            }
        }
        parser.lexer.seek(StringToken::Quotation).map_err(Pos::inherit)?;
        Ok(string.into())
    }
}

impl std::fmt::Display for EscapedStringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "\""),
            Self::ReverseSolidus => write!(f, "\\"),
            Self::Solidus => write!(f, "/"),
            Self::Backspace => write!(f, "b"),
            Self::Formfeed => write!(f, "f"),
            Self::Linefeed => write!(f, "n"),
            Self::CarriageReturn => write!(f, "r"),
            Self::HorizontalTab => write!(f, "t"),
            Self::Unicode(_) => write!(f, "u"),
        }
    }
}
impl LL1Token for EscapedStringToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            '"' => Ok(Self::Quotation),
            '\\' => Ok(Self::ReverseSolidus),
            '/' => Ok(Self::Solidus),
            'b' => Ok(Self::Backspace),
            'f' => Ok(Self::Formfeed),
            'n' => Ok(Self::Linefeed),
            'r' => Ok(Self::CarriageReturn),
            't' => Ok(Self::HorizontalTab),
            'u' => Ok(Self::Unicode(EscapedUnicodeToken::Hex4Digits(LL1::Lookahead(Default::default())))),
            &c => Err(TokenizeError::UnexpectedEscape { escaped: c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        Self::lookahead(&LL1::ahead(s)?).map_err(|_| TokenizeError::UnmatchedToken { s: s.into() })
    }
}
impl JsonToken for EscapedStringToken {
    type Output = char;
    type Error = Positioned<ParseStringError<ValueToken>>;
    /// parse `escape_sequence` of json. the following ebnf is not precise.<br>
    /// `escape_sequence` := "\\"" | "\\\\" | "\\/" | "\n" | "\r" | "\t" | `unicode`
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let ((start, _), _reverse_solidus) = parser.lexer.seek(StringToken::ReverseSolidus).map_err(Pos::inherit)?;
        match parser.lexer.branch() {
            Ok(EscapedStringToken::Unicode(_)) => EscapedUnicodeToken::parse(parser),
            Ok(t) => {
                parser.lexer.seek(t.clone()).expect("previous lookahead ensure this seek success"); // TODO do not use clone
                Ok(t.try_into().map_err(|e| Pos::with(e, start, parser.lexer.pos()))?)
            }
            Err(e) => Err(Pos(e).cast::<LexerError<StringToken>>())?,
        }
    }
}
impl From<EscapedStringToken> for StringToken {
    fn from(token: EscapedStringToken) -> Self {
        Self::Escaped(token)
    }
}
impl From<LexerError<EscapedStringToken>> for LexerError<StringToken> {
    fn from(value: LexerError<EscapedStringToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}
impl TryFrom<EscapedStringToken> for char {
    type Error = ParseStringError<ValueToken>; // TODO not use ParseStringError
    fn try_from(token: EscapedStringToken) -> Result<Self, Self::Error> {
        match &token {
            EscapedStringToken::Quotation => Ok('"'),
            EscapedStringToken::ReverseSolidus => Ok('\\'),
            EscapedStringToken::Solidus => Ok('/'),
            EscapedStringToken::Backspace | EscapedStringToken::Formfeed => {
                Err(ParseStringError::UnsupportedEscape { token: StringToken::from(token).into() })
            }
            EscapedStringToken::Linefeed => Ok('\n'),
            EscapedStringToken::CarriageReturn => Ok('\r'),
            EscapedStringToken::HorizontalTab => Ok('\t'),
            EscapedStringToken::Unicode(_) => {
                // TODO if can be converted
                Err(ParseStringError::CannotConvertChar { token: StringToken::from(token).into() })
            }
        }
    }
}

impl std::fmt::Display for EscapedUnicodeToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex4Digits(s) => s.fmt(f),
        }
    }
}
impl LL1Token for EscapedUnicodeToken {
    type Error = TokenizeError<Self>;
    type Symbol = TerminalSymbol;
    fn lookahead(c: &char) -> Result<Self, Self::Error> {
        match c {
            &c @ ('0'..='9' | 'a'..='f' | 'A'..='F') => Ok(Self::Hex4Digits(LL1::Lookahead(c))),
            &c => Err(TokenizeError::UnexpectedEscape { escaped: c }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        // TODO implement ?
        match Self::lookahead(&LL1::ahead(s)?) {
            Ok(EscapedUnicodeToken::Hex4Digits(LL1::Lookahead(c))) => {
                Ok(EscapedUnicodeToken::Hex4Digits(LL1::Tokenized(c.to_string())))
            }
            _ => Err(TokenizeError::UnmatchedToken { s: s.into() }),
        }
    }
}
impl JsonToken for EscapedUnicodeToken {
    type Output = char;
    type Error = Positioned<ParseStringError<ValueToken>>;
    /// parse `unicode` of json. the following ebnf is not precise.<br>
    /// `unicode` := "\u" `hex4digits`
    fn parse(parser: &mut crate::syntax::parser::Parser) -> Result<Self::Output, <Self as JsonToken>::Error> {
        let mut hex4 = String::new();
        let ((_start, _), _reverse_solidus) = parser
            .lexer
            .seek(EscapedStringToken::Unicode(EscapedUnicodeToken::Hex4Digits(LL1::Lookahead(Default::default()))))
            .map_err(|e| Pos(e).cast::<LexerError<StringToken>>())?;
        for _ in 0..4 {
            match parser.lexer.branch() {
                Ok(t @ EscapedUnicodeToken::Hex4Digits(_)) => {
                    hex4.push_str(&parser.lexer.seek(t).expect("previous lookahead ensure this seek success").1)
                }
                Err(e) => Err(Pos(e).cast::<LexerError<StringToken>>())?,
            }
        }
        Ok(unsafe {
            char::from_u32_unchecked(
                u32::from_str_radix(&hex4, 16).expect("previous lookahead ensure this convert success"),
            )
        })
    }
}
impl From<EscapedUnicodeToken> for EscapedStringToken {
    fn from(token: EscapedUnicodeToken) -> Self {
        EscapedStringToken::Unicode(token)
    }
}
impl From<EscapedUnicodeToken> for StringToken {
    fn from(token: EscapedUnicodeToken) -> Self {
        Self::Escaped(token.into())
    }
}
impl From<LexerError<EscapedUnicodeToken>> for LexerError<StringToken> {
    fn from(value: LexerError<EscapedUnicodeToken>) -> Self {
        // TODO this cause stack overflow?
        value.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{syntax::parser::Parser, Value};

    #[test]
    fn test_tokenize() {
        assert!(matches!(StringToken::lookahead(&'"'), Ok(StringToken::Quotation)));
        assert!(matches!(StringToken::lookahead(&'{'), Ok(StringToken::Unescaped(LL1::Lookahead('{')))));
        assert!(matches!(StringToken::lookahead(&'\\'), Ok(StringToken::ReverseSolidus)));
        assert!(matches!(EscapedStringToken::lookahead(&'n'), Ok(EscapedStringToken::Linefeed)));
        assert!(matches!(EscapedStringToken::lookahead(&'f'), Ok(EscapedStringToken::Formfeed)));
        assert!(matches!(
            EscapedUnicodeToken::lookahead(&'A'),
            Ok(EscapedUnicodeToken::Hex4Digits(LL1::Lookahead('A')))
        ));
        assert!(matches!(EscapedUnicodeToken::lookahead(&'Z'), Err(_)));

        assert!(matches!(StringToken::tokenize("\""), Ok(StringToken::Quotation)));
        assert!(matches!(StringToken::tokenize("\\"), Ok(StringToken::ReverseSolidus)));
        assert!(matches!(EscapedStringToken::tokenize("\""), Ok(EscapedStringToken::Quotation)));
        assert!(matches!(EscapedStringToken::tokenize("n"), Ok(EscapedStringToken::Linefeed)));
        assert_eq!(
            EscapedUnicodeToken::tokenize("7").unwrap(),
            EscapedUnicodeToken::Hex4Digits(LL1::Tokenized("7".to_string()))
        );
        assert!(matches!(EscapedUnicodeToken::tokenize("Z"), Err(_)));
    }

    #[test]
    fn test_parse_string() {
        let unescaped = r#""Rust""#.into();
        let mut parser = Parser::new(&unescaped);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("Rust".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 6), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let quotation = r#""Ru\"st""#.into();
        let mut parser = Parser::new(&quotation);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("Ru\"st".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let solidus = r#""Ru\/st""#.into();
        let mut parser = Parser::new(&solidus);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("Ru/st".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let solidus = r#""Ru\nst""#.into();
        let mut parser = Parser::new(&solidus);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("Ru\nst".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 8), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let unicode = r#""R\u00f9st""#.into();
        let mut parser = Parser::new(&unicode);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("Rùst".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 11), '\n')));
        assert_eq!(parser.lexer.next(), None);

        let unicode = r#""\u01a6\u03Cd\u03E8\u01aC""#.into();
        let mut parser = Parser::new(&unicode);
        let string = StringToken::parse(&mut parser).unwrap();
        assert_eq!(string, Value::String("ƦύϨƬ".to_string()));
        assert_eq!(parser.lexer.next(), Some(((0, 26), '\n')));
        assert_eq!(parser.lexer.next(), None);
    }
}
