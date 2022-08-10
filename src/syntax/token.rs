use std::fmt::{Debug, Display};

pub trait Token: PartialEq + Eq + Display + Debug {
    fn tokenize(c: char) -> Self;
}

#[derive(PartialEq, Eq, Debug)]
pub enum MainToken {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Quotation,
    Digit,
    Plus,
    Minus,
    Dot,
    Whitespace,
    Undecided(char),
}
impl std::fmt::Display for MainToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "LeftBrace({{)"),
            Self::RightBrace => write!(f, "RightBrace(}})"),
            Self::LeftBracket => write!(f, "LeftBracket([)"),
            Self::RightBracket => write!(f, "RightBracket(])"),
            Self::Colon => write!(f, "Colon(:)"),
            Self::Comma => write!(f, "Comma(,)"),
            Self::Quotation => write!(f, "Quotation(\")"),
            Self::Digit => write!(f, "Digit(0-9)"),
            Self::Plus => write!(f, "Plus(+)"),
            Self::Minus => write!(f, "Minus(-)"),
            Self::Dot => write!(f, "Dot(.)"),
            Self::Whitespace => write!(f, "Whitespace( )"),
            Self::Undecided(c) => write!(f, "Undecided({c})"),
        }
    }
}
impl Token for MainToken {
    fn tokenize(c: char) -> Self {
        match c {
            '{' => Self::LeftBrace,
            '}' => Self::RightBrace,
            '[' => Self::LeftBracket,
            ']' => Self::RightBracket,
            ':' => Self::Colon,
            ',' => Self::Comma,
            '"' => Self::Quotation,
            '0'..='9' => Self::Digit,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '.' => Self::Dot,
            ' ' | '\n' | '\r' | '\t' => Self::Whitespace,
            c => Self::Undecided(c),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
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
    Unexpected(char),
}
impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "Quotation(\")"),
            Self::ReverseSolidus => write!(f, "ReverseSolidus(\\)"),
            Self::Solidus => write!(f, "Solidus(/)"),
            Self::Backspace => write!(f, "Backspace(\\b)"),
            Self::Formfeed => write!(f, "Formfeed(\\f)"),
            Self::Linefeed => write!(f, "Linefeed(\\n)"),
            Self::CarriageReturn => write!(f, "CarriageReturn(\\r)"),
            Self::HorizontalTab => write!(f, "HorizontalTab(\\t)"),
            Self::Unicode => write!(f, "Unicode(\\u)"),
            Self::Unexpected(c) => write!(f, "Unexpected({c})"),
        }
    }
}
impl Token for StringToken {
    fn tokenize(c: char) -> Self {
        match c {
            '"' => Self::Quotation,
            '\\' => Self::ReverseSolidus,
            '/' => Self::Solidus,
            'b' => Self::Backspace,
            'f' => Self::Formfeed,
            'n' => Self::Linefeed,
            'r' => Self::CarriageReturn,
            't' => Self::HorizontalTab,
            'u' => Self::Unicode,
            c => Self::Unexpected(c),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum NumberToken {
    Zero,
    OneNine,
    Plus,
    Minus,
    Dot,
    Exponent,
    Unexpected(char),
}
impl std::fmt::Display for NumberToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero(0)"),
            Self::OneNine => write!(f, "OneNine(1-9)"),
            Self::Plus => write!(f, "Plus(+)"),
            Self::Minus => write!(f, "Minus(-)"),
            Self::Dot => write!(f, "Dot(.)"),
            Self::Exponent => write!(f, "Exponent(e|E)"),
            Self::Unexpected(c) => write!(f, "Unexpected({c})"),
        }
    }
}
impl Token for NumberToken {
    fn tokenize(c: char) -> Self {
        match c {
            '0' => Self::Zero,
            '1'..='9' => Self::OneNine,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '.' => Self::Dot,
            'e' | 'E' => Self::Exponent,
            c => Self::Unexpected(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undecided() {
        assert_eq!(MainToken::tokenize('t'), MainToken::Undecided('t'));
        assert_eq!(MainToken::tokenize('f'), MainToken::Undecided('f'));
        assert_eq!(MainToken::tokenize('n'), MainToken::Undecided('n'));
    }
}
