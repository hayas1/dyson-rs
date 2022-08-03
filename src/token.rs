use std::fmt::Display;

pub trait TokenType: Display + PartialEq {
    fn token_type(c: char) -> Self;
    fn is_whitespace(c: char) -> bool;
    fn is_start_object(c: char) -> bool;
    fn is_start_array(c: char) -> bool;
    fn is_start_bool(c: char) -> bool;
    fn is_start_null(c: char) -> bool;
    fn is_start_number(c: char) -> bool;
}

#[derive(PartialEq, Eq, Debug)]
pub enum SimpleToken {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Quotation,
    ReverseSolidus,
    Digit,
    Sign,
    Dot,
    Exponent,
    Whitespace,
    Undecided,
}

impl Display for SimpleToken {
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
            Self::Digit => write!(f, "Digit(0-9)"),
            Self::Sign => write!(f, "Sign(+ or -)"),
            Self::Dot => write!(f, "Dot(.)"),
            Self::Exponent => write!(f, "Exponent(e or E)"),
            Self::Whitespace => write!(f, "Whitespace( )"),
            Self::Undecided => write!(f, "Undecided(???)"),
        }
    }
}

impl TokenType for SimpleToken {
    fn token_type(c: char) -> Self {
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
            '+' | '-' => Self::Sign,
            '.' => Self::Dot,
            'e' | 'E' => Self::Exponent,
            ' ' | '\n' | '\r' | '\t' => Self::Whitespace,
            _ => Self::Undecided,
        }
    }

    fn is_whitespace(c: char) -> bool {
        Self::token_type(c) == Self::Whitespace
    }

    fn is_start_object(c: char) -> bool {
        Self::token_type(c) == Self::LeftBrace
    }

    fn is_start_array(c: char) -> bool {
        Self::token_type(c) == Self::LeftBracket
    }

    fn is_start_bool(c: char) -> bool {
        matches!(c, 't' | 'f')
    }

    fn is_start_null(c: char) -> bool {
        matches!(c, 'n')
    }

    fn is_start_number(c: char) -> bool {
        matches!(c, '0'..='9')
    }
}
