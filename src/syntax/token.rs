use super::error::TokenizeError;

// TODO remove unused trait bound
pub trait LL1Token: PartialEq + std::fmt::Display + std::fmt::Debug + Send + Sync + Sized {
    type Error: std::fmt::Display + std::fmt::Debug + Send + Sync;
    fn lookahead(c: char) -> Result<Self, Self::Error>;
    fn tokenize(s: &str) -> Result<Self, Self::Error>;
    fn is_whitespace(c: char) -> bool {
        matches!(JsonToken::lookahead(c), Ok(JsonToken::Whitespace))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JsonToken {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    // Quotation,
    // Plus,
    // Minus,
    // Dot,
    Whitespace,
    Number(NumberToken),
    Immediate(ImmediateToken),
    String(EscapedStringToken),
}

impl std::fmt::Display for JsonToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            // Self::Quotation => write!(f, "\""),
            // Self::Plus => write!(f, "+"),
            // Self::Minus => write!(f, "-"),
            // Self::Dot => write!(f, "."),
            Self::Whitespace => write!(f, " "),
            Self::Number(t) => write!(f, "{}", t),
            Self::Immediate(t) => write!(f, "{}", t),
            Self::String(t) => write!(f, "{}", t),
        }
    }
}
impl LL1Token for JsonToken {
    type Error = TokenizeError;
    fn lookahead(c: char) -> Result<Self, Self::Error> {
        match c {
            '{' => Ok(Self::LeftBrace),
            '}' => Ok(Self::RightBrace),
            '[' => Ok(Self::LeftBracket),
            ']' => Ok(Self::RightBracket),
            ':' => Ok(Self::Colon),
            ',' => Ok(Self::Comma),
            // '"' => Ok(Self::Quotation),
            // '+' => Ok(Self::Plus),
            // '-' => Ok(Self::Minus),
            // '.' => Ok(Self::Dot),
            ' ' | '\n' | '\r' | '\t' => Ok(Self::Whitespace),
            '+' | '-' | '.' | '0'..='9' => Ok(Self::Number(NumberToken::lookahead(c)?)),
            '"' => Ok(Self::String(EscapedStringToken::lookahead(c)?)),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        Err(TokenizeError::InvalidTokenize { s: s.to_string(), token_type: std::any::type_name::<Self>().to_string() })
    }
}

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
impl LL1Token for ImmediateToken {
    type Error = TokenizeError;
    fn lookahead(c: char) -> Result<Self, Self::Error> {
        match c {
            't' => Ok(Self::True),
            'f' => Ok(Self::False),
            'n' => Ok(Self::Null),
            _ => Err(TokenizeError::UnmatchedTokenPrefix { c, token_type: std::any::type_name::<Self>().to_string() }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        match s {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "null" => Ok(Self::Null),
            _ => Err(TokenizeError::UnmatchedToken { s: s.to_string() }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EscapedStringToken {
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
}
impl std::fmt::Display for EscapedStringToken {
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
            Self::Hex4Digits(s) => write!(f, "{}", s),
        }
    }
}
impl LL1Token for EscapedStringToken {
    type Error = TokenizeError;
    fn lookahead(c: char) -> Result<Self, Self::Error> {
        match c {
            '"' => Ok(Self::Quotation),
            '\\' => Ok(Self::ReverseSolidus),
            '/' => Ok(Self::Solidus),
            'b' => Ok(Self::Backspace),
            'f' => Ok(Self::Formfeed),
            'n' => Ok(Self::Linefeed),
            'r' => Ok(Self::CarriageReturn),
            't' => Ok(Self::HorizontalTab),
            'u' => Ok(Self::Unicode),
            '0'..='9' | 'a'..='f' | 'A'..='F' => Ok(Self::Hex4Digits(Default::default())),
            _ => Err(TokenizeError::UnmatchedTokenPrefix { c, token_type: std::any::type_name::<Self>().to_string() }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        // match s {
        //     "\\\"" => Ok(Self::Quotation),
        //     "\\\\" => Ok(Self::ReverseSolidus),
        //     "/" => Ok(Self::Solidus),
        //     "\\b" => Ok(Self::Backspace),
        //     "\\f" => Ok(Self::Formfeed),
        //     "\\n" => Ok(Self::Linefeed),
        //     "\\r" => Ok(Self::CarriageReturn),
        //     "\\t" => Ok(Self::HorizontalTab),
        //     "\\u" => Ok(Self::Unicode),
        //     _ if s.len() == 4 && u32::from_str_radix(s, 16).is_ok() => Ok(Self::Hex4Digits(s.to_string())),
        //     _ => Err(TokenizeError::UnmatchedToken { s: s.to_string() }),
        // }
        Err(TokenizeError::InvalidTokenize { s: s.to_string(), token_type: std::any::type_name::<Self>().to_string() })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumberToken {
    Zero,
    OneNine(char),
    Plus,
    Minus,
    Dot,
    Exponent,
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
        }
    }
}
impl LL1Token for NumberToken {
    type Error = TokenizeError;
    fn lookahead(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Zero),
            '1'..='9' => Ok(Self::OneNine(c)),
            '+' => Ok(Self::Plus),
            '-' => Ok(Self::Minus),
            '.' => Ok(Self::Dot),
            'e' | 'E' => Ok(Self::Exponent),
            _ => Err(TokenizeError::UnmatchedTokenPrefix { c, token_type: std::any::type_name::<Self>().to_string() }),
        }
    }
    fn tokenize(s: &str) -> Result<Self, Self::Error> {
        Err(TokenizeError::InvalidTokenize { s: s.to_string(), token_type: std::any::type_name::<Self>().to_string() })
    }
}

// TODO implement
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenWithComment {
    MainToken(JsonToken),
    DoubleSlash,
    CommentContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookahead() {
        assert!(matches!(JsonToken::lookahead('{'), Ok(JsonToken::RightBrace)));
        assert!(matches!(JsonToken::lookahead('f'), Ok(JsonToken::Immediate(ImmediateToken::False))));
        assert!(matches!(NumberToken::lookahead('+'), Ok(NumberToken::Plus)));
    }
}
