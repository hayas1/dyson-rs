use super::{
    error::{LexerError, Pos, Positioned},
    rawjson::RawJson,
    token::{LL1Token, SkipWhiteSpace},
    Position,
};

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
        let curr = json.get(0, 0).map(|&c| ((0, 0), c));
        Self { json, curr }
    }

    pub fn cursor(&mut self, (row, col): (usize, usize)) -> Option<&<Self as Iterator>::Item> {
        self.curr = self.json.get(row, col).map(|&c| ((row, col), c));
        self.peek()
    }

    /// peek next token without skip whitespace. this method's complexity is **O(1)**.
    /// if next token is eof, return None.
    pub fn peek(&self) -> Option<&<Self as Iterator>::Item> {
        self.curr.as_ref()
    }

    /// read next token with skip whitespace. this method's complexity is **O(len(ws))**, but first call of this method
    /// will move cursor to end of whitespace, so consecutive call of this method will be **O(1)** complexity.
    pub fn skip_whitespace<T: LL1Token>(&mut self) -> Option<&<Self as Iterator>::Item> {
        while let Some((_, c)) = self.peek() {
            if T::Symbol::whitespace(c) {
                self.next();
            } else {
                break;
            }
        }
        self.peek()
    }

    /// move cursor to previous next token
    pub fn stick<T: LL1Token>(&mut self) -> Option<&<Self as Iterator>::Item> {
        if T::Symbol::skip_ws() {
            self.skip_whitespace::<T>()
        } else {
            self.peek()
        }
    }

    /// get position of lexer cursor
    pub fn pos(&self) -> Position {
        self.peek().map_or_else(|| self.json.eof(), |&(pos, _)| pos)
    }

    /// peek next 1 char, and decide type of token
    pub fn branch<T: LL1Token>(&mut self) -> Result<T, Positioned<LexerError<T>>> {
        if let Some((_, c)) = self.stick::<T>() {
            Ok(T::lookahead(c)
                .map_err(|error| Pos::with(LexerError::FailedLookahead { error }, self.pos(), self.pos()))?)
        } else {
            Err(Pos::with(LexerError::LookaheadEof {}, self.pos(), self.pos()))?
        }
    }

    /// read expected token and move cursor after it
    pub fn lex<T: LL1Token>(
        &mut self,
        expected: T,
    ) -> Result<((Position, Position), String), Positioned<LexerError<T>>> {
        if let Some(&(start, _)) = self.stick::<T>() {
            let s = self.take(expected.to_string().len()).map(|(_, c)| c).collect::<String>();
            match T::tokenize(&s) {
                Ok(td) if td == expected => Ok(((start, self.pos()), s)),
                Ok(found) => Err(Pos::with(LexerError::UnexpectedToken { found, expected }, start, self.pos()))?,
                Err(error) => Err(Pos::with(LexerError::FailedTokenize { expected, error }, start, self.pos()))?,
            }
        } else {
            Err(Pos::with(LexerError::UnexpectedEof { expected }, self.pos(), self.pos()))?
        }
    }

    /// read expected token and move cursor after it. if unexpected token is found, return cursor before position.
    pub fn seek<T: LL1Token>(
        &mut self,
        expected: T,
    ) -> Result<((Position, Position), String), Positioned<LexerError<T>>> {
        let stuck = self.stick::<T>().cloned();
        let result = self.lex(expected);
        match (stuck, result) {
            (_, lexed @ Ok(_)) => lexed,
            (Some((start, _)), error @ Err(_)) => {
                self.cursor(start);
                error
            }
            (_, error) => error,
        }
    }

    // /// read next expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    // /// if success, lexer cursor move to next, but if error, lexer cursor do not move next (skip whitespace only).
    // pub fn lex_1_char<T>(&mut self, expected: T) -> anyhow::Result<<Self as Iterator>::Item>
    // where
    //     T: 'static + std::fmt::Debug + LL1Token,
    // {
    //     if let Some((pos, c)) = if T::Symbol::skip_ws() { self.skip_whitespace() } else { self.peek() } {
    //         match T::lookahead(c) {
    //             Some(t) if t == expected => self.next().ok_or_else(|| unreachable!("ensured by previous peek")),
    //             Some(token) => Err(LexError::UnexpectedToken { token, expected, pos })?,
    //             None => Err(LexError::UnexpectedLookahead { found: c, pos })?,
    //         }
    //     } else {
    //         Err(LexError::UnexpectedEof { expected, pos: self.json.eof() })?
    //     }
    // }

    // /// read next `n` chars ***without*** skipping whitespace until white space. this method's complexity is **O(n)**.
    // /// if success, lexer cursor move `n` step, but if error, lexer cursor will stop error ocurred position.
    // pub fn lex_n_chars(&mut self, n: usize) -> anyhow::Result<(String, Option<<Self as Iterator>::Item>)> {
    //     // HACK this functions is only used by unicode parsing (and parse immediate), so should be refactored
    //     if n == 0 {
    //         return Ok((String::new(), self.peek().cloned()));
    //     }
    //     let &(start, _) = self.peek().ok_or_else(|| LexTokenError::EofWhileLex::<JsonToken> {
    //         found: "".into(),
    //         start: self.json.eof(),
    //         end: self.json.eof(),
    //     })?;
    //     let mut result = String::new();
    //     for (p, c) in self.take(n) {
    //         if JsonToken::is_whitespace(c) {
    //             return Err(LexTokenError::UnexpectedWhiteSpace::<JsonToken> { found: result, start, end: p })?;
    //         } else {
    //             result.push(c)
    //         }
    //     }
    //     if result.len() == n {
    //         Ok((result, self.peek().cloned()))
    //     } else {
    //         Err(LexTokenError::EofWhileLex::<JsonToken> { found: result, start, end: self.json.eof() })?
    //     }
    // }

    // /// read next sequential token with skipping whitespace until line separator.
    // /// this method's complexity is **O(len(token))** (see [lex_n_chars](Lexer)).
    // pub fn lex_expected<T>(&mut self, expected: T) -> anyhow::Result<Option<<Self as Iterator>::Item>>
    // where
    //     T: 'static + std::fmt::Debug + LL1Token,
    // {
    //     // HACK this functions is only used by parse immediate, so should be refactored
    //     if let Some(&(start, _)) = self.skip_whitespace() {
    //         let (ts, nexted) = self.lex_n_chars(expected.to_string().len())?;
    //         match T::tokenize(&ts) {
    //             Ok(t) if t == expected => Ok(nexted),
    //             Ok(token) => Err(LexTokenError::UnexpectedToken { token, expected, pos: start })?,
    //             Err(error) => Err(LexTokenError::TokenizeError::<T> { error, pos: start })?,
    //         }
    //     } else {
    //         Err(LexTokenError::UnexpectedEof { expected, pos: self.json.eof() })?
    //     }
    // }

    // /// peek next token is equal to expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    // pub fn is_next<T: LL1Token>(&mut self, expected: T) -> bool {
    //     if T::Symbol::skip_ws() { self.skip_whitespace() } else { self.peek() }
    //         .map(|&(_p, c)| matches!(T::lookahead(c), Ok(e) if e == expected))
    //         .unwrap_or(false)
    // }
}

#[cfg(test)]
mod tests {
    use crate::syntax::token::{object::ObjectToken, string::StringToken, value::ValueToken};

    use super::*;

    #[test]
    fn test_json_read() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut lexer = json.lexer();
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
        let json = vec!["{", "    \"a\": 1", "}"].into_iter().collect();
        let expected = vec!['{', '"', 'a', '"', ':', '1', '}'];
        let (mut i, mut lexer) = (0, Lexer::new(&json));
        assert_eq!(lexer.skip_whitespace::<ValueToken>(), Some(&((0, 0), '{')));
        assert_eq!(lexer.skip_whitespace::<ValueToken>(), Some(&((0, 0), '{')));
        assert_eq!(lexer.skip_whitespace::<ValueToken>(), Some(&((0, 0), '{')));
        assert_eq!(lexer.skip_whitespace::<ValueToken>(), Some(&((0, 0), '{')));
        assert_eq!(lexer.skip_whitespace::<ValueToken>(), Some(&((0, 0), '{')));
        while lexer.skip_whitespace::<ValueToken>().is_some() {
            assert_eq!(lexer.next().unwrap().1, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_seek() {
        let json = r#"{"a": 123}"#.into();
        let mut lexer = Lexer::new(&json);
        let (pos, left_brace) = lexer.seek(ObjectToken::LeftBrace).unwrap();
        assert_eq!((pos, left_brace), (((0, 0), (0, 1)), "{".to_string()));
        let (pos, quotation) = lexer.seek(StringToken::Quotation).unwrap();
        assert_eq!((pos, quotation), (((0, 1), (0, 2)), "\"".to_string()));
    }

    // #[test]
    // fn test_lex_1_char() {
    //     let json = vec![" {", " ]"].into_iter().collect();
    //     let mut lexer = Lexer::new(&json);
    //     let error = lexer.lex_1_char::<_, SkipWs<false>>(JsonToken::LeftBrace).unwrap_err();
    //     println!("{}", error);
    //     assert!(error.to_string().contains(&postr(&(0, 0))));
    //     assert!(error.to_string().contains("Whitespace"));
    //     assert!(error.to_string().contains("LeftBrace"));
    //     let ok = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::LeftBrace).unwrap();
    //     assert_eq!(ok, ((0, 1), '{'));
    //     let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
    //     assert!(error.to_string().contains(&postr(&(1, 1))));
    //     assert!(error.to_string().contains('}'));
    //     assert!(error.to_string().contains(']'));
    //     assert!(lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
    //     assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
    //     let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
    //     assert!(error.to_string().contains(&postr(&(1, 1))));
    //     assert!(error.to_string().contains('}'));
    //     assert!(error.to_string().contains(']'));
    //     assert!(lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
    //     assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
    //     let ok = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBracket).unwrap();
    //     assert_eq!(ok, ((1, 1), ']'));
    //     let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
    //     assert!(error.to_string().to_lowercase().contains("eof"));
    //     assert!(error.to_string().contains('}'));
    //     assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
    //     assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
    // }

    // #[test]
    // fn test_lex_n_chars() {
    //     let json = "[true,  fal\nse]".into();
    //     let mut lexer = Lexer::new(&json);
    //     assert_eq!(lexer.next(), Some(((0, 0), '[')));
    //     let (lex_4_chars, nexted) = lexer.lex_n_chars(4).unwrap();
    //     assert_eq!(lex_4_chars, "true");
    //     assert_eq!(nexted, Some(((0, 5), ',')));
    //     assert_eq!(lexer.next(), Some(((0, 5), ',')));
    //     assert_eq!(lexer.skip_whitespace(), Some(&((0, 8), 'f')));
    //     let lex_5_chars = lexer.lex_n_chars(5).unwrap_err();
    //     assert!(lex_5_chars.to_string().contains("fal"));
    // }
}
