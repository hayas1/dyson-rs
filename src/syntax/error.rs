use super::{token::SequentialToken, SingleToken, StringToken};
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
pub enum ParseStringError {
    #[error("{} - {}: unexpected Linefeed, cannot close string literal \"{}\"", postr(start), postr(end), comp)]
    UnexpectedLinefeed { comp: String, start: Position, end: Position },

    #[error("{} - {}: unexpected EOF, cannot close string literal \"{}\"", postr(start), postr(end), comp)]
    UnexpectedEof { comp: String, start: Position, end: Position },

    #[error("{} - {}: unsupported {:?} in Rust", postr(start), postr(end), escape)]
    UnsupportedEscapeSequence { escape: StringToken, start: Position, end: Position },

    #[error("{} - {}: {} cannot be converted into unicode", postr(start), postr(end), uc)]
    CannotConvertUnicode { uc: String, start: Position, end: Position },

    #[error("{} - {}: unexpected escape sequence {:?}", postr(start), postr(end), escape)]
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
    // use super::*;

    #[test]
    fn test_parse_invalid_object() {
        // TODO
    }
}
