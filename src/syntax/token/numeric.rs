#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumericToken {
    Zero,
    OneNine(char),
    Plus,
    Minus,
    Dot,
    Exponent, // TODO distinguish `E` from `e` ?
}
impl std::fmt::Display for NumericToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::OneNine(c) => write!(f, "{}", c),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Dot => write!(f, "."),
            Self::Exponent => write!(f, "E"),
        }
    }
}
