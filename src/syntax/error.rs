use thiserror::Error;

use super::{
    token::{numeric::NumericToken, string::StringToken, LL1Token},
    Position,
};

pub(crate) fn postr((row, col): &Position) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}

pub(crate) fn join_token<'a, I: IntoIterator<Item = &'a T>, T: 'a + LL1Token>(iter: I, sep: &str) -> String {
    // let res = iter.into_iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(sep);
    // if res.is_empty() {
    //     "some token".to_string()
    // } else {
    //     res
    // }
    todo!()
}

#[derive(Error, Debug)]
#[error("{} - {}: {:?}", postr(start), postr(end), source)]
pub struct ParserError<T: std::fmt::Debug + Send + Sync> {
    #[source]
    source: T,
    start: Position,
    end: Position,
}
pub trait WithPos: std::fmt::Debug + Send + Sync + Sized {
    fn with_pos(self, start: Position, end: Position) -> ParserError<Self>;
}
impl<E: std::fmt::Debug + Send + Sync> WithPos for E {
    fn with_pos(self, start: Position, end: Position) -> ParserError<Self> {
        ParserError { start, end, source: self }
    }
}
impl<T: LL1Token> From<ParserError<LexerError<T>>> for ParserError<ParseObjectError<T>> {
    fn from(error: ParserError<LexerError<T>>) -> Self {
        ParserError { source: error.source.into(), start: error.start, end: error.end }
    }
}
impl<T: LL1Token> From<ParserError<LexerError<T>>> for ParserError<ParseStringError<T>> {
    fn from(error: ParserError<LexerError<T>>) -> Self {
        ParserError { source: error.source.into(), start: error.start, end: error.end }
    }
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("`{}` has {} length, so cannot convert to char", s, s.len())]
    TooLongString { s: String },

    #[error("cannot convert empty string to char")]
    EmptyString,
}

#[derive(Error, Debug)]
pub enum TokenizeError<T: LL1Token> {
    #[error("`{}` seem to be not json's token", s)]
    UnmatchedToken { s: String },

    #[error("cannot tokenize `{}` as `{}`", s, std::any::type_name::<T>())]
    InvalidTokenize { s: String },

    #[error("no `{}` token start with `{}` ", std::any::type_name::<T>(), c)]
    UnmatchedTokenPrefix { c: char },

    #[error("{}", error)]
    ConvertError { error: ConvertError },

    #[error("expected {:?}, but found {:?}", expected, token)]
    UnexpectedToken { token: T, expected: T },
}
impl<T: LL1Token> From<ConvertError> for TokenizeError<T> {
    fn from(error: ConvertError) -> Self {
        Self::ConvertError { error }
    }
}

#[derive(Error, Debug)]
pub enum LexerError<T: LL1Token> {
    #[error("{:?}", error)]
    FailedLookahead { error: T::Error },

    #[error("cannot lookahead, found eof")]
    LookaheadEof {},

    #[error("expected {:?}, but found {:?}", expected, found)]
    UnexpectedToken { found: T, expected: T },

    #[error("expected {:?}, but error occurred \"{:?}\"", expected, error)]
    FailedTokenize { expected: T, error: T::Error },

    #[error("expected {:?}, but found EOF", expected)]
    UnexpectedEof { expected: T },
}
impl<T: LL1Token> From<LexerError<T>> for ParseObjectError<T> {
    fn from(error: LexerError<T>) -> Self {
        Self::LexError { error }
    }
}
impl<T: LL1Token> From<LexerError<T>> for ParseArrayError<T> {
    fn from(error: LexerError<T>) -> Self {
        Self::LexError { error }
    }
}
impl<T: LL1Token> From<LexerError<T>> for ParseImmediateError<T> {
    fn from(error: LexerError<T>) -> Self {
        Self::LexError { error }
    }
}
impl<T: LL1Token> From<LexerError<T>> for ParseStringError<T> {
    fn from(error: LexerError<T>) -> Self {
        Self::LexError { error }
    }
}
impl<T: LL1Token> From<LexerError<T>> for ParseNumericError<T> {
    fn from(error: LexerError<T>) -> Self {
        Self::LexError { error }
    }
}

#[derive(Error, Debug)]
pub enum ParseObjectError<T: LL1Token> {
    #[error("{}", error)]
    LexError { error: LexerError<T> },

    #[error("{}", error)]
    ParseStringError { error: ParseStringError<T> },
}
impl<T: LL1Token> From<ParseStringError<T>> for ParseObjectError<T> {
    fn from(error: ParseStringError<T>) -> Self {
        Self::ParseStringError { error }
    }
}

#[derive(Error, Debug)]
pub enum ParseArrayError<T: LL1Token> {
    #[error("{}", error)]
    LexError { error: LexerError<T> },
}

#[derive(Error, Debug)]
pub enum ParseImmediateError<T: LL1Token> {
    #[error("{}", error)]
    LexError { error: LexerError<T> },
}

#[derive(Error, Debug)]
pub enum ParseStringError<T: LL1Token> {
    #[error("{} - {}: unexpected Linefeed, cannot close string literal \"{}\"", postr(start), postr(end), comp)]
    UnexpectedLinefeed { comp: String, start: Position, end: Position },

    #[error("{} - {}: unexpected EOF, cannot close string literal \"{}\"", postr(start), postr(end), comp)]
    UnexpectedEof { comp: String, start: Position, end: Position },

    #[error("{} - {}: unsupported {:?} in Rust", postr(start), postr(end), escape)]
    UnsupportedEscapeSequence { escape: StringToken, start: Position, end: Position },

    #[error("{} - {}: {} cannot be converted into unicode", postr(start), postr(end), uc)]
    CannotConvertUnicode { uc: String, start: Position, end: Position },

    #[error("{} - {}: unexpected escape sequence \"\\{}\"", postr(start), postr(end), escape)]
    UnexpectedEscapeSequence { escape: StringToken, start: Position, end: Position },

    #[error("{}", error)]
    LexError { error: LexerError<T> },
}

#[derive(Error, Debug)]
pub enum ParseNumericError<T: LL1Token> {
    #[error("{} - {}: unexpected EOF, cannot close string literal \"{}\"", postr(start), postr(end), num)]
    UnexpectedEof { num: String, start: Position, end: Position },

    // #[error(
    //     "{}: expected leading value token such as {}, but found {:?}",
    //     postr(pos),
    //     join_token(expected, " or "),
    //     found
    // )]
    // UnexpectedToken { expected: Vec<NumericToken>, found: NumericToken, pos: Position },
    #[error("{} - {}: \"{}\" maybe valid number, but cannot be converted into `i64`", postr(start), postr(end), num)]
    CannotConvertI64 { num: String, start: Position, end: Position },

    #[error("{} - {}: \"{}\" maybe valid number, but cannot be converted into `f64`", postr(start), postr(end), num)]
    CannotConvertF64 { num: String, start: Position, end: Position },

    #[error("{}: empty digits is not allowed", postr(pos))]
    EmptyDigits { pos: Position },

    #[error("{}", error)]
    LexError { error: LexerError<T> },
}

#[derive(Error, Debug)] // TODO pos -> start, end
pub enum StructureError {
    #[error("{}: trailing comma is not allowed in json", postr(pos))]
    TrailingComma { pos: Position },

    #[error("{} - {}: found surplus token previous EOF", postr(start), postr(end))]
    FoundSurplus { start: Position, end: Position },
}

#[cfg(test)]
mod tests {
    use crate::ast::Value;

    #[test]
    fn test_parse_empty() {
        let empty = "";
        let err = Value::parse(empty).unwrap_err();
        assert!(err.to_string().contains("token"));
    }

    #[test]
    fn test_parse_double() {
        let double = "{{}}";
        let err = Value::parse(double).unwrap_err();
        assert!(err.to_string().contains('}'));
        assert!(err.to_string().contains('{'));

        let double = "[[]]";
        let ok = Value::parse(double).unwrap();
        assert_eq!(ok, Value::Array(vec![Value::Array(Vec::new())]));
    }

    #[test]
    fn test_trailing_comma() {
        let object = r#"
        {
            "one": 1,
            "two": 2,
        }
        "#;
        let err = Value::parse(object).unwrap_err();
        assert!(err.to_string().contains("trailing comma"));

        let array = r#"
        [
            "one",
            "two",
        ]
        "#;
        let err = Value::parse(array).unwrap_err();
        assert!(err.to_string().contains("trailing comma"));
    }

    #[test]
    fn test_invalid_string() {
        let open = "\"not closed string";
        let err = Value::parse(open).unwrap_err();
        assert!(err.to_string().contains("cannot close string"));

        let invalid_es = "\"\\d mean digit\"";
        let err = Value::parse(invalid_es).unwrap_err();
        assert!(err.to_string().contains("unexpected escape sequence"));
        assert!(err.to_string().contains("\\d"));

        let unsupported_es = "\"formfeed \\f is not supported\"";
        let err = Value::parse(unsupported_es).unwrap_err();
        assert!(err.to_string().contains("unsupported"));
        assert!(err.to_string().to_lowercase().contains("formfeed"));
        assert!(err.to_string().contains("\\f"));

        let quotation = r#"
        {
            "one": 1,
            "two": "three"four",
        }"#;
        let err = Value::parse(quotation).unwrap_err();
        // println!("{err}"); // this is not good message
        assert!(err.to_string().contains('}'));
        assert!(err.to_string().contains(','));
    }

    #[test]
    fn test_invalid_number() {
        let prefix_plus = "+123";
        let err = Value::parse(prefix_plus).unwrap_err();
        assert!(err.to_string().contains('+'));

        let dot2 = "1.2.3";
        let err = Value::parse(dot2).unwrap_err();
        assert!(err.to_string().contains("found"));

        let ee = "1eE5";
        let err = Value::parse(ee).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("exponent"));

        let overflow = "999999999999999999999999999999999999999999999999999999999999";
        let err = Value::parse(overflow).unwrap_err();
        assert!(err.to_string().contains("maybe valid number"));
    }

    #[test]
    fn test_invalid_json() {
        let rs = r#"
        {
            1: "one",
            2: "two"
        }"#;
        let err = Value::parse(rs).unwrap_err();
        // println!("{err}"); // this is not good message
        assert!(err.to_string().contains('}'));
        assert!(err.to_string().contains(','));
    }

    #[test]
    fn test_surplus_json() {
        let rs = r#"
        {
            "one": 1,
            "two": 2
        }, "this is text"#;
        let err = Value::parse(rs).unwrap_err();
        assert!(err.to_string().contains("surplus"));
    }

    #[test]
    fn test_invalid_value() {
        let rs = "invalid json";
        let err = Value::parse(rs).unwrap_err();
        assert!(err.to_string().contains("value"));
    }
}
