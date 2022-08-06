use std::fmt::Display;

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
    Digit,
    Plus,
    Minus,
    Dot,
    Exponent,
    Whitespace,
    Undecided(char),
}
impl Display for Token {
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
            Self::Plus => write!(f, "Plus(+)"),
            Self::Minus => write!(f, "Minus(-)"),
            Self::Dot => write!(f, "Dot(.)"),
            Self::Exponent => write!(f, "Exponent(e or E)"),
            Self::Whitespace => write!(f, "Whitespace( )"),
            Self::Undecided(c) => write!(f, "Undecided({c})"),
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
            'e' | 'E' => Self::Exponent,
            ' ' | '\n' | '\r' | '\t' => Self::Whitespace,
            c => Self::Undecided(c),
        }
    }
}
