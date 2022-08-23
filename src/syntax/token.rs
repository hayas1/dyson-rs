use std::fmt::{Debug, Display};

pub trait SingleToken: PartialEq + Eq + Display + Debug + Clone + Send + Sync {
    fn tokenize(c: char) -> Self;
}
pub trait SequentialToken: SingleToken {
    fn confirm(s: &str) -> Self;
    fn tokenize(c: char) -> Self {
        SingleToken::tokenize(c)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum MainToken {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Quotation,
    Digit(char),
    Plus,
    Minus,
    Dot,
    Whitespace,
    Eof,
    Undecided(char),
}
impl std::fmt::Display for MainToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainToken::LeftBrace => write!(f, "{{"),
            MainToken::RightBrace => write!(f, "}}"),
            MainToken::LeftBracket => write!(f, "["),
            MainToken::RightBracket => write!(f, "]"),
            MainToken::Colon => write!(f, ":"),
            MainToken::Comma => write!(f, ","),
            MainToken::Quotation => write!(f, "\""),
            MainToken::Digit(c) => write!(f, "{}", c),
            MainToken::Plus => write!(f, "+"),
            MainToken::Minus => write!(f, "-"),
            MainToken::Dot => write!(f, "."),
            MainToken::Whitespace => write!(f, " "),
            MainToken::Eof => write!(f, "\\0"),
            MainToken::Undecided(c) => write!(f, "{}", c),
        }
    }
}
impl std::fmt::Debug for MainToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "LeftBrace({})", self),
            Self::RightBrace => write!(f, "RightBrace({})", self),
            Self::LeftBracket => write!(f, "LeftBracket({})", self),
            Self::RightBracket => write!(f, "RightBracket({})", self),
            Self::Colon => write!(f, "Colon({})", self),
            Self::Comma => write!(f, "Comma({})", self),
            Self::Quotation => write!(f, "Quotation({})", self),
            Self::Digit(_) => write!(f, "Digit({})", self),
            Self::Plus => write!(f, "Plus({})", self),
            Self::Minus => write!(f, "Minus({})", self),
            Self::Dot => write!(f, "Dot({})", self),
            Self::Whitespace => write!(f, "Whitespace({})", self),
            Self::Eof => write!(f, "Eof({})", self),
            Self::Undecided(_) => write!(f, "Undecided({})", self),
        }
    }
}
impl SingleToken for MainToken {
    fn tokenize(c: char) -> Self {
        match c {
            '{' => Self::LeftBrace,
            '}' => Self::RightBrace,
            '[' => Self::LeftBracket,
            ']' => Self::RightBracket,
            ':' => Self::Colon,
            ',' => Self::Comma,
            '"' => Self::Quotation,
            '0'..='9' => Self::Digit(c),
            '+' => Self::Plus,
            '-' => Self::Minus,
            '.' => Self::Dot,
            ' ' | '\n' | '\r' | '\t' => Self::Whitespace,
            c => Self::Undecided(c),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ImmediateToken {
    True,
    False,
    Null,
    Undecided(char),
    Unexpected(String),
}
impl std::fmt::Display for ImmediateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImmediateToken::True => write!(f, "true"),
            ImmediateToken::False => write!(f, "false"),
            ImmediateToken::Null => write!(f, "null"),
            ImmediateToken::Undecided(c) => write!(f, "{}", c),
            ImmediateToken::Unexpected(s) => write!(f, "{}", s),
        }
    }
}
impl std::fmt::Debug for ImmediateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "True({})", self),
            Self::False => write!(f, "False({})", self),
            Self::Null => write!(f, "Null({})", self),
            Self::Undecided(_) => write!(f, "Undecided({})", self),
            Self::Unexpected(_) => write!(f, "Unexpected({})", self),
        }
    }
}
impl SingleToken for ImmediateToken {
    fn tokenize(c: char) -> Self {
        match c {
            't' | 'f' | 'n' => Self::Undecided(c),
            c => Self::Unexpected(c.to_string()),
        }
    }
}
impl SequentialToken for ImmediateToken {
    fn confirm(s: &str) -> Self {
        match s {
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            s => Self::Unexpected(s.to_string()),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
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
    Hex4Digits(String),
    Undecided(char),
    Unexpected(String),
}
impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringToken::Quotation => write!(f, "\""),
            StringToken::ReverseSolidus => write!(f, "\\"),
            StringToken::Solidus => write!(f, "/"),
            StringToken::Backspace => write!(f, "\\b"),
            StringToken::Formfeed => write!(f, "\\f"),
            StringToken::Linefeed => write!(f, "\\n"),
            StringToken::CarriageReturn => write!(f, "\\r"),
            StringToken::HorizontalTab => write!(f, "\\t"),
            StringToken::Unicode => write!(f, "\\u"),
            StringToken::Hex4Digits(s) => write!(f, "{}", s),
            StringToken::Undecided(s) => write!(f, "{}", s),
            StringToken::Unexpected(s) => write!(f, "{}", s),
        }
    }
}
impl std::fmt::Debug for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "Quotation({})", self),
            Self::ReverseSolidus => write!(f, "ReverseSolidus({})", self),
            Self::Solidus => write!(f, "Solidus({})", self),
            Self::Backspace => write!(f, "Backspace({})", self),
            Self::Formfeed => write!(f, "Formfeed({})", self),
            Self::Linefeed => write!(f, "Linefeed({})", self),
            Self::CarriageReturn => write!(f, "CarriageReturn({})", self),
            Self::HorizontalTab => write!(f, "HorizontalTab({})", self),
            Self::Unicode => write!(f, "Unicode({})", self),
            Self::Hex4Digits(_) => write!(f, "Unicode({})", self),
            Self::Undecided(_) => write!(f, "Undecided({})", self),
            Self::Unexpected(_) => write!(f, "Unexpected({})", self),
        }
    }
}
impl SingleToken for StringToken {
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
            '0'..='9' | 'a'..='f' | 'A'..='F' => Self::Undecided(c),
            c => Self::Unexpected(c.to_string()),
        }
    }
}
impl SequentialToken for StringToken {
    fn confirm(s: &str) -> Self {
        match s {
            "\\\"" => Self::Quotation,
            "\\\\" => Self::ReverseSolidus,
            "/" => Self::Solidus,
            "\\b" => Self::Backspace,
            "\\f" => Self::Formfeed,
            "\\n" => Self::Linefeed,
            "\\r" => Self::CarriageReturn,
            "\\t" => Self::HorizontalTab,
            "\\u" => Self::Unicode,
            _ if u32::from_str_radix(s, 16).is_ok() => Self::Hex4Digits(s.to_string()),
            s => Self::Unexpected(s.to_string()),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum NumberToken {
    Zero,
    OneNine(char),
    Plus,
    Minus,
    Dot,
    Exponent,
    Unexpected(char),
}
impl std::fmt::Display for NumberToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::OneNine(c) => write!(f, "{}", c),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Dot => write!(f, "."),
            Self::Exponent => write!(f, "E"),
            Self::Unexpected(c) => write!(f, "({})", c),
        }
    }
}
impl std::fmt::Debug for NumberToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero({})", self),
            Self::OneNine(_) => write!(f, "OneNine({})", self),
            Self::Plus => write!(f, "Plus({})", self),
            Self::Minus => write!(f, "Minus({})", self),
            Self::Dot => write!(f, "Dot({})", self),
            Self::Exponent => write!(f, "Exponent({})", self),
            Self::Unexpected(_) => write!(f, "Unexpected({})", self),
        }
    }
}
impl SingleToken for NumberToken {
    fn tokenize(c: char) -> Self {
        match c {
            '0' => Self::Zero,
            '1'..='9' => Self::OneNine(c),
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
        assert_eq!(<ImmediateToken as SingleToken>::tokenize('t'), ImmediateToken::Undecided('t'));
        assert_eq!(<ImmediateToken as SingleToken>::tokenize('f'), ImmediateToken::Undecided('f'));
        assert_eq!(<ImmediateToken as SingleToken>::tokenize('n'), ImmediateToken::Undecided('n'));
    }
}
