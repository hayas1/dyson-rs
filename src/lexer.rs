use std::marker::PhantomData;

use anyhow::{bail, ensure};

use crate::{
    json::RawJson,
    token::{ExtraToken, TokenType},
};

pub type Nexted = ((usize, usize), char); // next is not verb but...
pub type Peeked<'a> = &'a Nexted;

pub struct Lexer<'a, T> {
    json: &'a RawJson,
    curr: Option<((usize, usize), char)>,
    token: PhantomData<T>,
}
impl<'a, T> Iterator for Lexer<'a, T> {
    type Item = ((usize, usize), char);
    fn next(&mut self) -> Option<Self::Item> {
        let ((row, col), curr) = self.curr?;
        if col + 1 < self.json[row].len() {
            self.curr = Some(((row, col + 1), self.json[row][col + 1]));
        } else if row + 1 < self.json.rows() {
            self.curr = Some(((row + 1, 0), self.json[row + 1][0]));
        } else {
            self.curr = None;
        }
        Some(((row, col), curr))
    }
}
impl<'a, T: TokenType> Lexer<'a, T> {
    pub fn new(json: &'a RawJson) -> Self {
        let curr = (!json.is_empty()).then(|| ((0, 0), json[0][0]));
        Self {
            json,
            curr,
            token: PhantomData,
        }
    }

    pub fn peek(&self) -> Option<Peeked> {
        self.curr.as_ref()
    }

    pub fn skip_white_space(&mut self) -> Option<Peeked> {
        while let Some(&(_, c)) = self.peek() {
            if T::is_whitespace(c) {
                self.next();
            } else {
                break;
            }
        }
        self.peek()
    }

    pub fn lex1char(&mut self, token: T) -> anyhow::Result<Nexted> {
        if let Some(&((_row, _col), c)) = self.skip_white_space() {
            ensure!(T::token_type(c) == token, "expected {}, but {}", token, c);
            self.next()
                .ok_or_else(|| unreachable!("previous peek ensure this next success"))
        } else {
            bail!("unexpected EOF, but expected {}", token)
        }
    }
}
impl<'a> Lexer<'a, ExtraToken> {
    pub fn skip_line(&mut self) -> (Option<Peeked>, String) {
        // FIXME use early return?
        if let Some(&((start_row, _start_col), _c)) = self.peek() {
            let mut content = String::new();
            while let Some(&((row, _col), c)) = self.peek() {
                if start_row < row {
                    return (self.peek(), content);
                } else {
                    content.push(c);
                    self.next();
                }
            }
            (None, content)
        } else {
            (None, String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::SimpleToken;

    use super::*;

    #[test]
    fn test_json_read() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut lexer = Lexer::<SimpleToken>::new(&json);
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.next(), Some(((0, 0), '{')));
        assert_eq!(lexer.next(), Some(((1, 0), '"')));
        assert_eq!(lexer.next(), Some(((1, 1), 'a')));
        assert_eq!(lexer.next(), Some(((1, 2), '"')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.next(), Some(((1, 3), ':')));
        assert_eq!(lexer.next(), Some(((1, 4), ' ')));
        assert_eq!(lexer.next(), Some(((1, 5), '1')));
        assert_eq!(lexer.next(), Some(((2, 0), '}')));
        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.next(), None);
        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.next(), None);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_skip_whitespace() {
        let json: RawJson = vec!["{", "    \"a\": 1", "}"].into_iter().collect();
        let expected = vec!['{', '"', 'a', '"', ':', '1', '}'];
        let (mut i, mut lexer) = (0, Lexer::<SimpleToken>::new(&json));
        while lexer.skip_white_space().is_some() {
            assert_eq!(lexer.next().unwrap().1, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_skip_line() {
        let json: RawJson = vec![
            "{",
            "# this is extra comment",
            "\"a\": 1",
            "}",
            "# comment previous EOF",
        ]
        .into_iter()
        .collect();
        let mut lexer = Lexer::<ExtraToken>::new(&json);
        assert_eq!(lexer.next(), Some(((0, 0), '{')));
        assert_eq!(
            lexer.skip_line(),
            (Some(&((2, 0), '"')), "# this is extra comment".to_string())
        );
        assert_eq!(lexer.next(), Some(((2, 0), '"')));
        assert_eq!(lexer.next(), Some(((2, 1), 'a')));
        assert_eq!(lexer.next(), Some(((2, 2), '"')));
        assert_eq!(lexer.next(), Some(((2, 3), ':')));
        assert_eq!(lexer.next(), Some(((2, 4), ' ')));
        assert_eq!(lexer.next(), Some(((2, 5), '1')));
        assert_eq!(lexer.next(), Some(((3, 0), '}')));
        assert_eq!(
            lexer.skip_line(),
            (None, "# comment previous EOF".to_string())
        );
    }
}
