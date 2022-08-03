use std::marker::PhantomData;

use anyhow::{bail, ensure};

use crate::{json::RawJson, token::TokenType};

pub struct Lexer<'a, T> {
    json: &'a RawJson,
    curr: Option<((usize, usize), char)>,
    token: PhantomData<T>,
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

    pub fn next(&mut self) -> Option<((usize, usize), char)> {
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

    pub fn peek(&self) -> Option<&((usize, usize), char)> {
        self.curr.as_ref()
    }

    pub fn skip_white_space(&mut self) -> Option<((usize, usize), char)> {
        while let Some(&(_, c)) = self.peek() {
            if T::is_whitespace(c) {
                self.next();
            } else {
                break;
            }
        }
        self.next()
    }

    pub fn lex1char(&mut self, token: T) -> anyhow::Result<((usize, usize), char)> {
        if let Some(((row, col), c)) = self.skip_white_space() {
            ensure!(T::token_type(c) == token, "expected {}, but {}", token, c);
            Ok(((row, col), c))
        } else {
            bail!("reach end of file")
        }
    }
}

// we do not want to filter or fold the lexer.
// impl<'a> Iterator for Lexer<'a> {
//     type Item = ((usize, usize), char);
//     fn next(&mut self) -> Option<Self::Item> {
//         self.next()
//     }
// }

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
        while let Some((_, char)) = lexer.skip_white_space() {
            assert_eq!(char, expected[i]);
            i += 1;
        }
    }
}
