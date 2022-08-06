use anyhow::{anyhow, bail, ensure};

use crate::{json::RawJson, postr, token::Token};

pub type Nexted = ((usize, usize), char); // next is not verb but...
pub type Peeked<'a> = &'a Nexted;

pub struct Lexer<'a> {
    json: &'a RawJson,
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
    pub fn new(json: &'a RawJson) -> Self {
        let curr = (!json.is_empty()).then(|| ((0, 0), json[0][0]));
        Self { json, curr }
    }

    pub fn peek(&self) -> Option<Peeked> {
        self.curr.as_ref()
    }

    pub fn skip_white_space(&mut self) -> Option<Peeked> {
        while let Some(&(_, c)) = self.peek() {
            if Token::tokenize(c) == Token::Whitespace {
                self.next();
            } else {
                break;
            }
        }
        self.peek()
    }

    /// read next expected token with skipping whitespace. this method's complexity is **O(len(ws))**.
    pub fn lex_1_char(&mut self, token: Token) -> anyhow::Result<Nexted> {
        if let Some(&(pos, c)) = self.skip_white_space() {
            ensure!(
                Token::tokenize(c) == token,
                "{}: unexpected {c}, but expected '{token}'",
                postr(pos)
            );
            self.next().ok_or_else(|| unreachable!("previous peek ensure this next success"))
        } else {
            bail!("unexpected EOF, but expected {token}",)
        }
    }

    /// read next `n` chars ***without*** skipping whitespace until line separator. this method's complexity is **O(n)**.
    pub fn lex_n_chars(&mut self, n: usize) -> anyhow::Result<String> {
        if n == 0 {
            return Ok(String::new());
        }
        let &((sr, sl), _c) =
            self.peek().ok_or_else(|| anyhow!("unexpected EOF, lex {n} chars"))?;
        let mut result = String::new();
        for _ in 0..n {
            let ((r, _l), c) = self.next().ok_or_else(|| {
                anyhow!("{}: unexpected EOF, unknown \"{result}\"", postr((sr, sl)))
            })?;
            (sr >= r).then(|| result.push(c)).ok_or_else(|| {
                anyhow!("{}: unexpected line separator, unknown \"{result}\"", postr((sr, sl)))
            })?;
        }
        Ok(result)
    }

    pub fn is_next(&mut self, token: Token) -> bool {
        self.skip_white_space().map(|&(_p, c)| Token::tokenize(c) == token).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_json_read() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut lexer = Lexer::new(&json);
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
        let (mut i, mut lexer) = (0, Lexer::new(&json));
        while lexer.skip_white_space().is_some() {
            assert_eq!(lexer.next().unwrap().1, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_lex_1_char() {
        let json: RawJson = vec!["{", "]"].into_iter().collect();
        let mut lexer = Lexer::new(&json);
        let ok = lexer.lex_1_char(Token::LeftBrace).unwrap();
        assert_eq!(ok, ((0, 0), '{'));
        let error = lexer.lex_1_char(Token::RightBrace).unwrap_err();
        assert!(error.to_string().contains(&postr((1, 0))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next(Token::RightBracket));
        assert!(!lexer.is_next(Token::RightBrace));
        let error = lexer.lex_1_char(Token::RightBrace).unwrap_err();
        assert!(error.to_string().contains(&postr((1, 0))));
        assert!(error.to_string().contains('}'));
        assert!(error.to_string().contains(']'));
        assert!(lexer.is_next(Token::RightBracket));
        assert!(!lexer.is_next(Token::RightBrace));
        let ok = lexer.lex_1_char(Token::RightBracket).unwrap();
        assert_eq!(ok, ((1, 0), ']'));
        let error = lexer.lex_1_char(Token::RightBrace).unwrap_err();
        assert!(error.to_string().to_lowercase().contains("eof"));
        assert!(error.to_string().contains('}'));
        assert!(!lexer.is_next(Token::RightBracket));
        assert!(!lexer.is_next(Token::RightBrace));
    }

    #[test]
    fn test_lex_n_chars() {
        let json = "[true,  fal\nse]".into();
        let mut lexer = Lexer::new(&json);
        assert_eq!(lexer.next(), Some(((0, 0), '[')));
        let lex_4_chars = lexer.lex_n_chars(4).unwrap();
        assert_eq!(lex_4_chars, "true");
        assert_eq!(lexer.next(), Some(((0, 5), ',')));
        assert_eq!(lexer.skip_white_space(), Some(&((0, 8), 'f')));
        let lex_5_chars = lexer.lex_n_chars(5).unwrap_err();
        assert!(lex_5_chars.to_string().contains("fal"));
    }
}
