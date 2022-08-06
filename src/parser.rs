use std::collections::HashMap;

use anyhow::{anyhow, bail};

use crate::{ast::Value, error_pos, lexer::Lexer, token::TokenType, ExtraToken, SimpleToken};

pub struct Parser<'a, T> {
    lexer: Lexer<'a, T>,
}

impl<'a> Parser<'a, SimpleToken> {
    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        if SimpleToken::is_start_object(c) {
            self.parse_object()
        } else if SimpleToken::is_start_array(c) {
            self.parse_array()
        } else if SimpleToken::is_start_immediate(c) {
            self.parse_immediate()
        } else {
            bail!(
                "{}: unexpected token \"{}\", while parse value",
                error_pos(pos),
                c
            )
        }
    }
    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Array(Vec::new()))
    }
    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Array(Vec::new()))
    }
}
impl<'a> Parser<'a, ExtraToken> {
    pub fn parse_value(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        if ExtraToken::is_start_object(c) {
            self.parse_object()
        } else if ExtraToken::is_start_array(c) {
            self.parse_array()
        } else if ExtraToken::is_start_immediate(c) {
            self.parse_immediate()
        } else if ExtraToken::is_start_comment(c) {
            self.lexer.skip_line();
            self.parse_value()
        } else {
            bail!(
                "{}: unexpected token \"{}\", while parse value",
                error_pos(pos),
                c
            )
        }
    }
    pub fn parse_object(&mut self) -> anyhow::Result<Value> {
        // TODO ExtraToken
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Object(HashMap::new()))
    }
    pub fn parse_array(&mut self) -> anyhow::Result<Value> {
        // TODO ExtraToken
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Array(Vec::new()))
    }
}

impl<'a, T: TokenType> Parser<'a, T> {
    pub fn parse_immediate(&mut self) -> anyhow::Result<Value> {
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        if T::is_start_bool(c) {
            self.parse_bool()
        } else if T::is_start_null(c) {
            self.parse_null()
        } else if T::is_start_number(c) {
            if let Ok(integer) = self.parse_integer() {
                Ok(integer)
            } else {
                self.parse_float()
            }
        } else if T::is_start_string(c) {
            self.parse_string()
        } else {
            bail!(
                "{}: unexpected token \"{}\", while parse immediate",
                error_pos(pos),
                c
            )
        }
    }

    pub fn parse_bool(&mut self) -> anyhow::Result<Value> {
        // TODO
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Bool(true))
    }

    pub fn parse_null(&mut self) -> anyhow::Result<Value> {
        // TODO
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Null)
    }

    pub fn parse_string(&mut self) -> anyhow::Result<Value> {
        // TODO
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::String("".to_string()))
    }

    pub fn parse_integer(&mut self) -> anyhow::Result<Value> {
        // TODO
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Integer(0))
    }
    pub fn parse_float(&mut self) -> anyhow::Result<Value> {
        // TODO
        let peeked = self.lexer.skip_white_space();
        let &(pos, c) = peeked.ok_or_else(|| anyhow!("unexpected EOF, while parse value"))?;
        Ok(Value::Float(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_object_parse() {
        let json = vec![
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json""#,
            r#"{"#,
        ];
    }
}
