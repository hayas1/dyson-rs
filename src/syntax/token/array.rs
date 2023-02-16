#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayToken {
    LeftBracket,
    RightBracket,
    Comma,
}

impl std::fmt::Display for ArrayToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Comma => write!(f, ","),
        }
    }
}
