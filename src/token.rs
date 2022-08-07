pub enum Context {
    ParseString,
    ParseNumber,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Quotation,
    ReverseSolidus,
    Solidus,
    Backspace,
    Formfeed,
    Linefeed,
    CarriageReturn,
    HorizontalTab,
    Unicode,
    Digit,
    Plus,
    Minus,
    Dot,
    Exponent,
    Whitespace,

    Undecided(char),
    Unexpected(char),
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "LeftBrace({{)"),
            Self::RightBrace => write!(f, "RightBrace(}})"),
            Self::LeftBracket => write!(f, "LeftBracket([)"),
            Self::RightBracket => write!(f, "RightBracket(])"),
            Self::Colon => write!(f, "Colon(:)"),
            Self::Comma => write!(f, "Comma(,)"),
            Self::Quotation => write!(f, "Quotation(\")"),
            Self::ReverseSolidus => write!(f, "ReverseSolidus(\\)"),
            Self::Solidus => write!(f, "Solidus(/)"),
            Self::Backspace => write!(f, "Backspace(\\b)"),
            Self::Formfeed => write!(f, "Formfeed(\\f)"),
            Self::Linefeed => write!(f, "Linefeed(\\n)"),
            Self::CarriageReturn => write!(f, "CarriageReturn(\\r)"),
            Self::HorizontalTab => write!(f, "HorizontalTab(\\t)"),
            Self::Unicode => write!(f, "Unicode(\\u)"),
            Self::Digit => write!(f, "Digit(0-9)"),
            Self::Plus => write!(f, "Plus(+)"),
            Self::Minus => write!(f, "Minus(-)"),
            Self::Dot => write!(f, "Dot(.)"),
            Self::Exponent => write!(f, "Exponent(e or E)"),
            Self::Whitespace => write!(f, "Whitespace( )"),
            Self::Undecided(c) => write!(f, "Undecided({c})"),
            Self::Unexpected(c) => write!(f, "Unexpected({c})"),
        }
    }
}
impl Token {
    pub fn tokenize(c: char) -> Self {
        match c {
            '{' => Self::LeftBrace,
            '}' => Self::RightBrace,
            '[' => Self::LeftBracket,
            ']' => Self::RightBracket,
            ':' => Self::Colon,
            ',' => Self::Comma,
            '"' => Self::Quotation,
            '\\' => Self::ReverseSolidus,
            '0'..='9' => Self::Digit,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '.' => Self::Dot,
            ' ' | '\n' | '\r' | '\t' => Self::Whitespace,
            c => Self::Undecided(c),
        }
    }

    pub fn tokenize_with_context(c: char, context: Option<Context>) -> Self {
        match context {
            Some(parsing) => match parsing {
                Context::ParseString => match c {
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
                },
                Context::ParseNumber => match c {
                    '0'..='9' => Self::Digit,
                    '+' => Self::Plus,
                    '-' => Self::Minus,
                    'e' | 'E' => Self::Exponent,
                    '.' => Self::Dot,
                    u => Self::Unexpected(u),
                },
            },
            None => Self::tokenize(c),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_undecided() {
        assert_eq!(Token::tokenize('t'), Token::Undecided('t'));
        assert_eq!(Token::tokenize('f'), Token::Undecided('f'));
        assert_eq!(Token::tokenize('n'), Token::Undecided('n'));
    }
}
