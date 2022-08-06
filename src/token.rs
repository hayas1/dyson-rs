use std::fmt::Display;

pub trait TokenType: Display + PartialEq {
    fn token_type(c: char) -> Self;
    fn is_whitespace(c: char) -> bool;
    fn is_start_object(c: char) -> bool;
    fn is_start_array(c: char) -> bool;
    fn is_start_immediate(c: char) -> bool;
    fn is_start_bool(c: char) -> bool;
    fn is_start_null(c: char) -> bool;
    fn is_start_string(c: char) -> bool;
    fn is_start_number(c: char) -> bool;
    fn is_end_number(c: char) -> bool;
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

    fn is_start_immediate(c: char) -> bool {
        Self::is_start_bool(c)
            || Self::is_start_null(c)
            || Self::is_start_string(c)
            || Self::is_start_number(c)
    }

    fn is_start_bool(c: char) -> bool {
        matches!(c, 't' | 'f')
    }

    fn is_start_null(c: char) -> bool {
        matches!(c, 'n')
    }

    fn is_start_string(c: char) -> bool {
        matches!(c, '"')
    }

    fn is_start_number(c: char) -> bool {
        matches!(c, '-' | '0'..='9')
    }

    fn is_end_number(c: char) -> bool {
        matches!(c, '0'..='9')
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExtraToken {
    SimpleToken(SimpleToken),
    Sharp,
}
impl Display for ExtraToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtraToken::SimpleToken(st) => st.fmt(f),
            ExtraToken::Sharp => write!(f, "Sharp(#)"),
        }
    }
}
impl ExtraToken {
    pub fn is_start_comment(c: char) -> bool {
        matches!(c, '#')
    }
}
impl TokenType for ExtraToken {
    fn token_type(c: char) -> Self {
        match c {
            '#' => Self::Sharp,
            _ => Self::SimpleToken(SimpleToken::token_type(c)),
        }
    }

    fn is_whitespace(c: char) -> bool {
        SimpleToken::is_whitespace(c)
    }

    fn is_start_object(c: char) -> bool {
        SimpleToken::is_start_object(c)
    }

    fn is_start_array(c: char) -> bool {
        SimpleToken::is_start_array(c)
    }

    fn is_start_immediate(c: char) -> bool {
        Self::is_start_bool(c)
            || Self::is_start_null(c)
            || Self::is_start_string(c)
            || Self::is_start_number(c)
    }

    fn is_start_bool(c: char) -> bool {
        SimpleToken::is_start_bool(c)
    }

    fn is_start_null(c: char) -> bool {
        SimpleToken::is_start_null(c)
    }

    fn is_start_string(c: char) -> bool {
        SimpleToken::is_start_string(c)
    }

    fn is_start_number(c: char) -> bool {
        matches!(c, '-' | '+' | '.' | '0'..='9')
    }

    fn is_end_number(c: char) -> bool {
        matches!(c, '.' | '0'..='9')
    }
}
