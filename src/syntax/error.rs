use super::token::{SequentialToken, SingleToken, StringToken};
use thiserror::Error;

pub type Position = (usize, usize);

pub(crate) fn postr((row, col): &Position) -> String {
    format!("line {} (col {})", row + 1, col + 1)
}

pub(crate) fn join_token<'a, I: IntoIterator<Item = &'a T>, T: 'a + SingleToken>(iter: I, sep: &str) -> String {
    let res = iter.into_iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join(sep);
    if res.is_empty() {
        "some token".to_string()
    } else {
        res
    }
}

#[derive(Error, Debug)]
pub enum SingleTokenError<T: SingleToken> {
    #[error("{}: expected {}, but found {:?}", postr(pos), join_token(expected, " or "), found)]
    UnexpectedToken { expected: Vec<T>, found: T, pos: Position },

    #[error("{}: expected {}, but found EOF", postr(pos), join_token(expected, " or "))]
    UnexpectedEof { expected: Vec<T>, pos: Position },
}

#[derive(Error, Debug)]
pub enum SequentialTokenError<T: SequentialToken> {
    #[error("{} - {}: expected {}, but found {:?}", postr(start), postr(end), join_token(expected, " or "), found)]
    UnexpectedToken { expected: Vec<T>, found: String, start: Position, end: Position },

    #[error("{} - {}: expected {}, but found EOF", postr(start), postr(end), join_token(expected, " or "))]
    UnexpectedEof { expected: Vec<T>, start: Position, end: Position },
}

#[derive(Error, Debug)]
pub enum ParseTokenError {
    #[error("{} - {}: unexpected EOF, unknown token \"{}\"", postr(start), postr(end), if found.is_empty() {"empty string"} else {found})]
    UnexpectedWhiteSpace { found: String, start: Position, end: Position },

    #[error("{} - {}: unexpected EOF, unknown token \"{}\"", postr(start), postr(end), if found.is_empty() {"empty string"} else {found})]
    UnexpectedEof { found: String, start: Position, end: Position },
}

#[derive(Error, Debug)]
pub enum StructureError {
    #[error("{}: trailing comma is not allowed in json", postr(pos))]
    TrailingComma { pos: Position },

    #[error("{} - {}: found surplus token previous EOF", postr(start), postr(end))]
    FoundSurplus { start: Position, end: Position },
}

#[derive(Error, Debug)]
pub enum ParseValueError<T: SingleToken> {
    #[error(
        "{}: expected leading value token such as {}, but found {:?}",
        postr(pos),
        join_token(examples, " or "),
        found
    )]
    CannotStartParseValue { examples: Vec<T>, found: T, pos: Position },
    #[error("{}: expected leading value token such as {}, but found EOF", postr(pos), join_token(examples, " or "))]
    UnexpectedEof { examples: Vec<T>, pos: Position },
}

#[derive(Error, Debug)]
pub enum ParseStringError {
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
}

#[derive(Error, Debug)]
pub enum ParseNumberError {
    #[error("{} - {}: unexpected EOF, cannot close string literal \"{}\"", postr(start), postr(end), num)]
    UnexpectedEof { num: String, start: Position, end: Position },

    #[error("{} - {}: \"{}\" maybe valid number, but cannot be converted into `i64`", postr(start), postr(end), num)]
    CannotConvertI64 { num: String, start: Position, end: Position },

    #[error("{} - {}: \"{}\" maybe valid number, but cannot be converted into `f64`", postr(start), postr(end), num)]
    CannotConvertF64 { num: String, start: Position, end: Position },

    #[error("{}: empty digits is not allowed", postr(pos))]
    EmptyDigits { pos: Position },
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
