use crate::syntax::error::{ParseStringError, Pos, Positioned, TokenizeError};

use super::{value::ValueToken, JsonToken, LL1Token, TerminalSymbol, LL1};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringToken {
    Quotation,
    ReverseSolidus,
    Solidus,
    Backspace,
    Formfeed,
    Linefeed,
    CarriageReturn,
    HorizontalTab,
    Unicode,
    Hex4Digits(LL1),
    Unescaped(LL1),
}
impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "\""),
            Self::ReverseSolidus => write!(f, "\\"),
            Self::Solidus => write!(f, "/"),
            Self::Backspace => write!(f, "\\b"),
            Self::Formfeed => write!(f, "\\f"),
            Self::Linefeed => write!(f, "\n"),
            Self::CarriageReturn => write!(f, "\r"),
            Self::HorizontalTab => write!(f, "\t"),
            Self::Unicode => write!(f, "\\u"),
            Self::Hex4Digits(c) => write!(f, "{}", c),
            Self::Unescaped(c) => write!(f, "{}", c),
        }
    }
}
impl TryFrom<StringToken> for char {
    type Error = ParseStringError<ValueToken>; // TODO not use ParseStringError
    fn try_from(token: StringToken) -> Result<Self, Self::Error> {
        match &token {
            StringToken::Quotation => Ok('"'),
            StringToken::ReverseSolidus => Ok('\\'),
            StringToken::Solidus => Ok('/'),
            StringToken::Backspace => Err(ParseStringError::UnsupportedEscape { token: token.into() }),
            StringToken::Formfeed => Err(ParseStringError::UnsupportedEscape { token: token.into() }),
            StringToken::Linefeed => Ok('\n'),
            StringToken::CarriageReturn => Ok('\r'),
            StringToken::HorizontalTab => Ok('\t'),
            StringToken::Unicode => Err(ParseStringError::CannotConvertChar { token: token.into() }),
            StringToken::Hex4Digits(_) => Err(ParseStringError::CannotConvertChar { token: token.into() }),
            StringToken::Unescaped(_) => Err(ParseStringError::CannotConvertChar { token: token.into() }), // TODO 1 length string
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
            '/' => Ok(Self::Solidus),
            'b' => Ok(Self::Backspace),
            'f' => Ok(Self::Formfeed),
            'n' => Ok(Self::Linefeed),
            'r' => Ok(Self::CarriageReturn),
            't' => Ok(Self::HorizontalTab),
            'u' => Ok(Self::Unicode),
            &c @ ('0'..='9' | 'a'..='f' | 'A'..='F') => Ok(Self::Hex4Digits(LL1::Lookahead(c))),
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
                Ok(StringToken::ReverseSolidus) => string.push(Self::parse_escaped(parser)?),
                Ok(_unescaped) => string.push(parser.lexer.next().expect("previous peek ensure this next success").1),
                Err(e) => Err(Pos::inherit(e))?,
            }
        }
        parser.lexer.seek(StringToken::Quotation).map_err(Pos::inherit)?;
        Ok(string.into())
    }
}
impl StringToken {
    /// parse `escape_sequence` of json. the following ebnf is not precise.<br>
    /// `escape_sequence` := "\\"" | "\\\\" | "\\/" | "\n" | "\r" | "\t" | `unicode`
    pub fn parse_escaped(parser: &mut crate::syntax::parser::Parser) -> Result<char, <Self as JsonToken>::Error> {
        let ((start, _), _reverse_solidus) = parser.lexer.seek(StringToken::ReverseSolidus).map_err(Pos::inherit)?;
        match parser.lexer.branch() {
            Ok(StringToken::Unicode) => Self::parse_unicode(parser),
            Ok(t) => {
                parser.lexer.seek(t.clone()).expect("previous lookahead ensure this seek success"); // TODO do not use clone
                Ok(t.try_into().map_err(|e| Pos::with(e, start, parser.lexer.pos()))?)
            }
            Err(e) => Err(Pos::inherit(e))?,
        }
    }

    /// parse `unicode` of json. the following ebnf is not precise.<br>
    /// `unicode` := "\u" `hex4digits`
    pub fn parse_unicode(parser: &mut crate::syntax::parser::Parser) -> Result<char, <Self as JsonToken>::Error> {
        todo!()
        // let (hex4, nexted) = lexer.lex_n_chars(4)?;
        // let (p, _) = nexted.ok_or_else(|| {
        //     let eof = lexer.json.eof();
        //     ParseStringError::UnexpectedEof { comp: hex4.clone(), start, end: eof }
        // })?;
        // let uc = char::from_u32(u32::from_str_radix(&hex4, 16)?);
        // Ok(uc.ok_or(ParseStringError::CannotConvertUnicode { uc: hex4, start, end: p })?)
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
        assert!(matches!(StringToken::lookahead(&'n'), Ok(StringToken::Linefeed)));
        assert!(matches!(StringToken::lookahead(&'f'), Ok(StringToken::Formfeed)));
        assert!(matches!(StringToken::tokenize("\""), Ok(StringToken::Quotation)));
        assert!(matches!(StringToken::tokenize("\\"), Ok(StringToken::ReverseSolidus)));
        assert!(matches!(StringToken::tokenize("n"), Ok(StringToken::Linefeed)));
        // assert!(matches!(StringToken::tokenize("\\n"), Ok(StringToken::Linefeed)));
        // assert!(matches!(StringToken::tokenize("\\n"), Ok(StringToken::Linefeed)));
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

        // let unicode = r#""R\u00f9st""#.into();
        // let (mut lexer, parser) = (Lexer::new(&unicode), Parser::new());
        // let unicode = parser.parse_string(&mut lexer).unwrap();
        // assert_eq!(unicode, Value::String("Rùst".to_string()));
        // assert_eq!(lexer.next(), Some(((0, 11), '\n')));
        // assert_eq!(lexer.next(), None);
    }
}
