#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImmediateToken {
    True,
    False,
    Null,
}
impl std::fmt::Display for ImmediateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
        }
    }
}
