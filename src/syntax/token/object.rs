#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ObjectToken {
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Whitespace,
}

impl std::fmt::Display for ObjectToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Whitespace => write!(f, " "),
        }
    }
}
