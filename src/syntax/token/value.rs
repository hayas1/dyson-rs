use super::{
    array::ArrayToken, immediate::ImmediateToken, numeric::NumericToken, object::ObjectToken, string::StringToken,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueToken {
    Object(ObjectToken),
    Array(ArrayToken),
    Immediate(ImmediateToken),
    String(StringToken),
    Numeric(NumericToken),
}
impl<'a> std::fmt::Display for ValueToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(t) => t.fmt(f),
            Self::Array(t) => t.fmt(f),
            Self::Immediate(t) => t.fmt(f),
            Self::String(t) => t.fmt(f),
            Self::Numeric(t) => t.fmt(f),
        }
    }
}
