use super::{
    error::ParseTokenError,
    token::{MainToken, SequentialToken, SingleToken},
};
use crate::{
    rawjson::RawJson,
    syntax::error::{SequentialTokenError, SingleTokenError},
};

pub type Nexted = ((usize, usize), char); // next is not verb but...
pub type Peeked<'a> = &'a Nexted;

pub struct Lexer<'a> {
    pub(crate) json: &'a RawJson,
    curr: Option<((usize, usize), char)>,
}
impl<'a> Iterator for Lexer<'a> {
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
impl<'a> Lexer<'a> {
    /// read next token without skip whitespace. this method's complexity is **O(1)**.
    /// if next token is eof, return None.
    pub fn new(json: &'a RawJson) -> Self {
        let curr = json.first();
        Self { json, curr }
    }

    /// peek next token without skip whitespace. this method's complexity is **O(1)**.
    /// if next token is eof, return None.
    pub fn peek(&self) -> Option<Peeked> {
        self.curr.as_ref()
    }

    /// read next token with skip whitespace. this method's complexity is **O(len(ws))**, but first call of this method
    /// will move cursor to end of whitespace, so consecutive call of this method will be **O(1)** complexity.
    pub fn skip_whitespace(&mut self) -> Option<Peeked> {
        while let Some(&(_, c)) = self.peek() {
            if MainToken::tokenize(c) == MainToken::Whitespace {
                self.next();
            } else {
                break;
            }
        }
        self.peek()
    }

    /// read next expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    /// if success, lexer cursor move to next, but if error, lexer cursor do not move next (skip whitespace only).
    pub fn lex_1_char<T: 'static + SingleToken>(&mut self, token: T, skip_ws: bool) -> anyhow::Result<Nexted> {
        if let Some(&(pos, c)) = if skip_ws { self.skip_whitespace() } else { self.peek() } {
            if T::tokenize(c) != token {
                Err(SingleTokenError::UnexpectedToken { expected: vec![token], found: T::tokenize(c), pos })?
            } else {
                self.next().ok_or_else(|| unreachable!("previous peek ensure this next success"))
            }
        } else {
            Err(SingleTokenError::UnexpectedEof { expected: vec![token], pos: self.json.eof() })?
        }
    }

    /// read next `n` chars ***without*** skipping whitespace until white space. this method's complexity is **O(n)**.
    /// if success, lexer cursor move `n` step, but if error, lexer cursor will stop error ocurred position.
    pub fn lex_n_chars(&mut self, n: usize) -> anyhow::Result<(String, Option<Nexted>)> {
        if n == 0 {
            return Ok((String::new(), self.peek().cloned()));
        }
        let &(start, _) = self.peek().ok_or_else(|| ParseTokenError::UnexpectedEof {
            found: "".into(),
            start: self.json.eof(),
            end: self.json.eof(),
        })?;
        let mut result = String::new();
        for (p, c) in self.take(n) {
            if MainToken::tokenize(c) == MainToken::Whitespace {
                return Err(ParseTokenError::UnexpectedWhiteSpace { found: result, start, end: p })?;
            } else {
                result.push(c)
            }
        }
        if result.len() == n {
            Ok((result, self.peek().cloned()))
        } else {
            Err(ParseTokenError::UnexpectedEof { found: result, start, end: self.json.eof() })?
        }
    }

    /// read next sequential token with skipping whitespace until line separator.
    /// this method's complexity is **O(len(token))** (see [lex_n_chars](Lexer)).
    pub fn lex_expected<T: 'static + SequentialToken>(&mut self, token: T) -> anyhow::Result<Option<Nexted>> {
        if let Some(&(start, _)) = self.skip_whitespace() {
            let (ts, nexted) = self.lex_n_chars(token.to_string().len())?;
            if T::confirm(&ts) == token {
                Ok(nexted)
            } else {
                let end = nexted.map(|(p, _)| p).unwrap_or_else(|| self.json.eof());
                Err(SequentialTokenError::UnexpectedToken { expected: vec![token], found: ts, start, end })?
            }
        } else {
            let eof = self.json.eof();
            Err(SequentialTokenError::UnexpectedEof { expected: vec![token], start: eof, end: eof })?
        }
    }

    /// peek next token is equal to expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    pub fn is_next<T: SingleToken>(&mut self, token: T, skip_ws: bool) -> bool {
        if skip_ws { self.skip_whitespace() } else { self.peek() }
            .map(|&(_p, c)| T::tokenize(c) == token)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::error::postr;

    use super::*;

    #[test]
    fn test_json_read() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut lexer = Lexer::new(&json);
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.peek(), Some(&((0, 0), '{')));
        assert_eq!(lexer.next(), Some(((0, 0), '{')));
        assert_eq!(lexer.next(), Some(((0, 1), '\n')));
        assert_eq!(lexer.next(), Some(((1, 0), '"')));
        assert_eq!(lexer.next(), Some(((1, 1), 'a')));
        assert_eq!(lexer.next(), Some(((1, 2), '"')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.peek(), Some(&((1, 3), ':')));
        assert_eq!(lexer.next(), Some(((1, 3), ':')));
        assert_eq!(lexer.next(), Some(((1, 4), ' ')));
        assert_eq!(lexer.next(), Some(((1, 5), '1')));
        assert_eq!(lexer.peek(), Some(&((1, 6), '\n')));
        assert_eq!(lexer.peek(), Some(&((1, 6), '\n')));
        assert_eq!(lexer.peek(), Some(&((1, 6), '\n')));
        assert_eq!(lexer.next(), Some(((1, 6), '\n')));
        assert_eq!(lexer.next(), Some(((2, 0), '}')));
        assert_eq!(lexer.peek(), Some(&((2, 1), '\n')));
        assert_eq!(lexer.next(), Some(((2, 1), '\n')));
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
        let (mut i, mut lexer) = (0, Lexer::new(&json));
        while lexer.skip_whitespace().is_some() {
            assert_eq!(lexer.next().unwrap().1, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_lex_1_char() {
        let json: RawJson = vec![" {", " ]"].into_iter().collect();
        let mut lexer = Lexer::new(&json);
        let error = lexer.lex_1_char(MainToken::LeftBrace, false).unwrap_err();
        println!("{}", error);
        assert!(error.to_string().contains(&postr(&(0, 0))));
        assert!(error.to_string().contains("Whitespace"));
        assert!(error.to_string().contains("LeftBrace"));
        let ok = lexer.lex_1_char(MainToken::LeftBrace, true).unwrap();
        assert_eq!(ok, ((0, 1), '{'));
        let error = lexer.lex_1_char(MainToken::RightBrace, true).unwrap_err();
        assert!(error.to_string().contains(&postr(&(1, 1))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next(MainToken::RightBracket, true));
        assert!(!lexer.is_next(MainToken::RightBrace, true));
        let error = lexer.lex_1_char(MainToken::RightBrace, true).unwrap_err();
        assert!(error.to_string().contains(&postr(&(1, 1))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next(MainToken::RightBracket, true));
        assert!(!lexer.is_next(MainToken::RightBrace, true));
        let ok = lexer.lex_1_char(MainToken::RightBracket, true).unwrap();
        assert_eq!(ok, ((1, 1), ']'));
        let error = lexer.lex_1_char(MainToken::RightBrace, true).unwrap_err();
        assert!(error.to_string().to_lowercase().contains("eof"));
        assert!(error.to_string().contains('}'));
        assert!(!lexer.is_next(MainToken::RightBracket, true));
        assert!(!lexer.is_next(MainToken::RightBrace, true));
    }

    #[test]
    fn test_lex_n_chars() {
        let json = "[true,  fal\nse]".into();
        let mut lexer = Lexer::new(&json);
        assert_eq!(lexer.next(), Some(((0, 0), '[')));
        let (lex_4_chars, nexted) = lexer.lex_n_chars(4).unwrap();
        assert_eq!(lex_4_chars, "true");
        assert_eq!(nexted, Some(((0, 5), ',')));
        assert_eq!(lexer.next(), Some(((0, 5), ',')));
        assert_eq!(lexer.skip_whitespace(), Some(&((0, 8), 'f')));
        let lex_5_chars = lexer.lex_n_chars(5).unwrap_err();
        assert!(lex_5_chars.to_string().contains("fal"));
    }
}
