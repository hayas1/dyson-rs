use super::{
    error::LexTokenError,
    rawjson::RawJson,
    token::{JsonToken, LL1Token},
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

    /// peek next token without skip whitespace. this method's complexity is **O(1)**.
    /// if next token is eof, return None.
    pub fn peek(&self) -> Option<&<Self as Iterator>::Item> {
        self.curr.as_ref()
    }

    /// read next token with skip whitespace. this method's complexity is **O(len(ws))**, but first call of this method
    /// will move cursor to end of whitespace, so consecutive call of this method will be **O(1)** complexity.
    pub fn skip_whitespace(&mut self) -> Option<&<Self as Iterator>::Item> {
        while let Some(&(_, c)) = self.peek() {
            if JsonToken::is_whitespace(c) {
                self.next();
            } else {
                break;
            }
        }
        self.peek()
    }

    /// read next expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    /// if success, lexer cursor move to next, but if error, lexer cursor do not move next (skip whitespace only).
    pub fn lex_1_char<T, S>(&mut self, expected: T) -> anyhow::Result<<Self as Iterator>::Item>
    where
        T: 'static + std::fmt::Debug + LL1Token,
        S: SkipWhiteSpace,
    {
        if let Some(&(pos, c)) = if S::skip_ws() { self.skip_whitespace() } else { self.peek() } {
            match T::lookahead(c) {
                Ok(t) if t == expected => self.next().ok_or_else(|| unreachable!("ensured by previous peek")),
                Ok(token) => Err(LexTokenError::UnexpectedToken { token, expected, pos })?,
                Err(error) => Err(LexTokenError::TokenizeError::<T> { error, pos })?,
            }
        } else {
            Err(LexTokenError::UnexpectedEof { expected, pos: self.json.eof() })?
        }
    }

    // TODO implement
    pub fn lex_until(&mut self) -> Option<<Self as Iterator>::Item> {
        // until whitespace of comma or parenthesis
        todo!()
    }

    /// read next `n` chars ***without*** skipping whitespace until white space. this method's complexity is **O(n)**.
    /// if success, lexer cursor move `n` step, but if error, lexer cursor will stop error ocurred position.
    pub fn lex_n_chars(&mut self, n: usize) -> anyhow::Result<(String, Option<<Self as Iterator>::Item>)> {
        // HACK this functions is only used by unicode parsing (and parse immediate), so should be refactored
        if n == 0 {
            return Ok((String::new(), self.peek().cloned()));
        }
        let &(start, _) = self.peek().ok_or_else(|| LexTokenError::EofWhileLex::<JsonToken> {
            found: "".into(),
            start: self.json.eof(),
            end: self.json.eof(),
        })?;
        let mut result = String::new();
        for (p, c) in self.take(n) {
            if JsonToken::is_whitespace(c) {
                return Err(LexTokenError::UnexpectedWhiteSpace::<JsonToken> { found: result, start, end: p })?;
            } else {
                result.push(c)
            }
        }
        if result.len() == n {
            Ok((result, self.peek().cloned()))
        } else {
            Err(LexTokenError::EofWhileLex::<JsonToken> { found: result, start, end: self.json.eof() })?
        }
    }

    /// read next sequential token with skipping whitespace until line separator.
    /// this method's complexity is **O(len(token))** (see [lex_n_chars](Lexer)).
    pub fn lex_expected<T>(&mut self, expected: T) -> anyhow::Result<Option<<Self as Iterator>::Item>>
    where
        T: 'static + std::fmt::Debug + LL1Token,
    {
        // HACK this functions is only used by parse immediate, so should be refactored
        if let Some(&(start, _)) = self.skip_whitespace() {
            let (ts, nexted) = self.lex_n_chars(expected.to_string().len())?;
            match T::tokenize(&ts) {
                Ok(t) if t == expected => Ok(nexted),
                Ok(token) => Err(LexTokenError::UnexpectedToken { token, expected, pos: start })?,
                Err(error) => Err(LexTokenError::TokenizeError::<T> { error, pos: start })?,
            }
        } else {
            Err(LexTokenError::UnexpectedEof { expected, pos: self.json.eof() })?
        }
    }

    /// peek next token is equal to expected token. if `skip_ws`, this method's complexity is **O(len(ws))** (see [skip_whitespace](Lexer)).
    pub fn is_next<T: LL1Token, S: SkipWhiteSpace>(&mut self, expected: T) -> bool {
        // HACK use peek instead of this
        if S::skip_ws() { self.skip_whitespace() } else { self.peek() }
            .map(|&(_p, c)| matches!(T::lookahead(c), Ok(e) if e == expected))
            .unwrap_or(false)
    }
}

pub struct SkipWs<const B: bool>;
pub trait SkipWhiteSpace {
    fn skip_ws() -> bool;
}
impl<const B: bool> SkipWhiteSpace for SkipWs<B> {
    fn skip_ws() -> bool {
        B
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::error::postr;

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
        while lexer.skip_whitespace().is_some() {
            assert_eq!(lexer.next().unwrap().1, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_lex_1_char() {
        let json = vec![" {", " ]"].into_iter().collect();
        let mut lexer = Lexer::new(&json);
        let error = lexer.lex_1_char::<_, SkipWs<false>>(JsonToken::LeftBrace).unwrap_err();
        println!("{}", error);
        assert!(error.to_string().contains(&postr(&(0, 0))));
        assert!(error.to_string().contains("Whitespace"));
        assert!(error.to_string().contains("LeftBrace"));
        let ok = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::LeftBrace).unwrap();
        assert_eq!(ok, ((0, 1), '{'));
        let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
        assert!(error.to_string().contains(&postr(&(1, 1))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
        assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
        let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
        assert!(error.to_string().contains(&postr(&(1, 1))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
        assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
        let ok = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBracket).unwrap();
        assert_eq!(ok, ((1, 1), ']'));
        let error = lexer.lex_1_char::<_, SkipWs<true>>(JsonToken::RightBrace).unwrap_err();
        assert!(error.to_string().to_lowercase().contains("eof"));
        assert!(error.to_string().contains('}'));
        assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBracket));
        assert!(!lexer.is_next::<_, SkipWs<true>>(JsonToken::RightBrace));
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
