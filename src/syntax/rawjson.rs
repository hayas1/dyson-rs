// use std::{ops::Index, slice::SliceIndex, vec::IntoIter};

/// [`RawJson`] represent raw json string sequence.
/// each sequence is terminated in line feed `'\n'`.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RawJson {
    json: Vec<Vec<char>>,
}

impl RawJson {
    /// return the number of rows. this method's complexity is **O(1)**.
    pub fn rows(&self) -> usize {
        self.json.len()
    }

    /// return eof position. this method's complexity is **O(1)**.
    pub fn eof(&self) -> (usize, usize) {
        let r = self.rows();
        if r > 0 {
            (r - 1, self.json[r - 1].len())
        } else {
            (0, 0)
        }
    }

    /// check is it empty. this method's complexity is **O(1)**.
    pub fn is_empty(&self) -> bool {
        self.rows() == 0
    }

    /// get first element and its position
    pub fn first(&self) -> Option<((usize, usize), char)> {
        (!self.is_empty()).then(|| ((0, 0), self[0][0]))
    }

    /// get iterator of raw json
    pub fn iter(&self) -> impl Iterator<Item = &Vec<char>> {
        self.json.iter()
    }
}

impl std::fmt::Display for RawJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().map(|l| l.iter().collect::<String>()).collect::<Vec<_>>().join("\n"))
    }
}

impl FromIterator<String> for RawJson {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            json: iter
                .into_iter()
                .flat_map(|s| s.replace("\r\n", "\n").split('\n').map(|s| s.to_string()).collect::<Vec<_>>())
                .map(|s| (s + "\n").chars().collect())
                .collect(),
        }
    }
}
impl<'a> FromIterator<&'a str> for RawJson {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        iter.into_iter().map(|s| s.to_string()).collect()
    }
}

impl From<String> for RawJson {
    fn from(s: String) -> Self {
        (&s[..]).into()
    }
}
impl From<&str> for RawJson {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Vec::<&str>::new().into_iter().collect()
        } else {
            s.replace("\r\n", "\n").split('\n').collect()
        }
    }
}

impl From<RawJson> for String {
    fn from(rj: RawJson) -> Self {
        rj.into_iter().map(|l| l.into_iter().collect::<String>()).collect::<Vec<_>>().join("\n")
    }
}

impl IntoIterator for RawJson {
    type Item = Vec<char>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.json.into_iter()
    }
}

impl<I: std::slice::SliceIndex<[Vec<char>]>> std::ops::Index<I> for RawJson {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.json[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_into_iter() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut j_iter = json.into_iter();
        let mut line1 = j_iter.next().unwrap().into_iter();
        assert_eq!(line1.next(), Some('{'));
        assert_eq!(line1.next(), Some('\n'));
        assert_eq!(line1.next(), None);
        let mut line2 = j_iter.next().unwrap().into_iter();
        assert_eq!(line2.next(), Some('"'));
        assert_eq!(line2.next(), Some('a'));
        assert_eq!(line2.next(), Some('"'));
        assert_eq!(line2.next(), Some(':'));
        assert_eq!(line2.next(), Some(' '));
        assert_eq!(line2.next(), Some('1'));
        assert_eq!(line2.next(), Some('\n'));
        assert_eq!(line2.next(), None);
        let mut line3 = j_iter.next().unwrap().into_iter();
        assert_eq!(line3.next(), Some('}'));
        assert_eq!(line3.next(), Some('\n'));
        assert_eq!(line3.next(), None);
        assert_eq!(j_iter.next(), None);
        // let _json_is_moved = json;  // compile error
    }

    #[test]
    fn test_json_iter() {
        let json: RawJson = "{\n\"b\": 2\r\n}".into();
        let expected = vec![vec!['{', '\n'], vec!['"', 'b', '"', ':', ' ', '2', '\n'], vec!['}', '\n']];
        for (l, el) in json.iter().zip(expected.iter()) {
            for (c, ec) in l.iter().zip(el.iter()) {
                assert_eq!(c, ec);
            }
        }
        let _json_is_not_moved = json; // not compile error
    }

    #[test]
    fn test_json_flatten() {
        let json: RawJson = vec!["{", "\"a\": 1", "}"].into_iter().collect();
        let mut j_iter = json.into_iter().flat_map(|l| l.into_iter());
        assert_eq!(j_iter.next(), Some('{'));
        assert_eq!(j_iter.next(), Some('\n'));
        assert_eq!(j_iter.next(), Some('"'));
        assert_eq!(j_iter.next(), Some('a'));
        assert_eq!(j_iter.next(), Some('"'));
        assert_eq!(j_iter.next(), Some(':'));
        assert_eq!(j_iter.next(), Some(' '));
        assert_eq!(j_iter.next(), Some('1'));
        assert_eq!(j_iter.next(), Some('\n'));
        assert_eq!(j_iter.next(), Some('}'));
        assert_eq!(j_iter.next(), Some('\n'));
        assert_eq!(j_iter.next(), None);
    }

    #[test]
    fn test_empty_json() {
        let json: RawJson = "".into();
        assert_eq!(json.rows(), 0);
        assert!(json.is_empty());
        let mut j_iter = json.into_iter();
        assert_eq!(j_iter.next(), None);
    }
}
