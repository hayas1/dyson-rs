use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::Index,
    path::Path,
    slice::SliceIndex,
    vec::IntoIter,
};

use anyhow::Ok;

pub struct RawJson {
    json: Vec<Vec<char>>,
}

impl RawJson {
    pub fn readfile(path: &Path) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        Self::read(BufReader::new(file))
    }

    pub fn read<B: BufRead>(bufread: B) -> anyhow::Result<Self> {
        let mut json = Vec::new();
        for line in bufread.lines() {
            json.push(line?.chars().collect())
        }
        Ok(Self { json })
    }

    pub fn rows(&self) -> usize {
        self.json.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows() == 0
    }
}

impl FromIterator<String> for RawJson {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            json: iter.into_iter().map(|s| s.chars().collect()).collect(),
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
        s.replace("\r\n", "\n")
            .split('\n')
            .filter(|l| !l.is_empty())
            .collect()
    }
}

impl From<RawJson> for String {
    fn from(rj: RawJson) -> Self {
        rj.into_iter()
            .map(|l| l.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl ToString for RawJson {
    fn to_string(&self) -> String {
        self.iter()
            .map(|l| l.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl IntoIterator for RawJson {
    type Item = Vec<char>;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.json.into_iter()
    }
}
impl RawJson {
    pub fn iter(&self) -> impl Iterator<Item = &Vec<char>> {
        self.json.iter()
    }
}

impl<I: SliceIndex<[Vec<char>]>> Index<I> for RawJson {
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
        let mut jiter = json.into_iter();
        let mut line1 = jiter.next().unwrap().into_iter();
        assert_eq!(line1.next(), Some('{'));
        assert_eq!(line1.next(), None);
        let mut line2 = jiter.next().unwrap().into_iter();
        assert_eq!(line2.next(), Some('"'));
        assert_eq!(line2.next(), Some('a'));
        assert_eq!(line2.next(), Some('"'));
        assert_eq!(line2.next(), Some(':'));
        assert_eq!(line2.next(), Some(' '));
        assert_eq!(line2.next(), Some('1'));
        assert_eq!(line2.next(), None);
        let mut line3 = jiter.next().unwrap().into_iter();
        assert_eq!(line3.next(), Some('}'));
        assert_eq!(line3.next(), None);
        assert_eq!(jiter.next(), None);
        // let _json_is_moved = json;  // compile error
    }

    #[test]
    fn test_json_iter() {
        let json: RawJson = "{\n\"b\": 2\r\n}".into();
        let expected = vec![vec!['{'], vec!['"', 'b', '"', ':', ' ', '2'], vec!['}']];
        for (l, el) in json.iter().zip(expected.iter()) {
            for (c, ec) in l.iter().zip(el.iter()) {
                assert_eq!(c, ec);
            }
        }
        let _json_is_not_moved = json; // not compile error
    }
}
