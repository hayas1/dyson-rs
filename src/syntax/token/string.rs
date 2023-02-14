#[derive(Debug, PartialEq, Eq, Clone)]
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
    // HexDigit(char),
    Unescaped(char),
}
impl std::fmt::Display for StringToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quotation => write!(f, "\""),
            Self::ReverseSolidus => write!(f, "\\"),
            Self::Solidus => write!(f, "/"),
            Self::Backspace => write!(f, "\\b"),
            Self::Formfeed => write!(f, "\\f"),
            Self::Linefeed => write!(f, "\\n"),
            Self::CarriageReturn => write!(f, "\\r"),
            Self::HorizontalTab => write!(f, "\\t"),
            Self::Unicode => write!(f, "\\u"),
            // Self::HexDigit(c) => write!(f, "{}", c),
            Self::Unescaped(c) => write!(f, "{}", c),
        }
    }
}
