use std::iter::FromIterator;

use itertools::Itertools;

use super::{
    index::{JsonIndex, JsonIndexer},
    quote, Value,
};

/// [`JsonPath`] is used for accessing [`Value`]. see [`Value::get`] also.
/// # examples
/// ```
/// use dyson::{ast::{index::JsonIndexer, path::JsonPath}, Value};
/// let raw_json = r#"{"key": [1, "two", 3, "four", 5]}"#;
/// let json = Value::parse(raw_json).unwrap();
///
/// let path =
///     vec![JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(0)].into_iter().collect::<JsonPath>();
/// assert_eq!(json[&path], Value::Integer(1));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPath {
    path: Vec<JsonIndexer>,
}

impl JsonIndex for &JsonPath {
    type Output = Value;
    fn gotten(self, value: &Value) -> Option<&Self::Output> {
        self.iter().fold(Some(value), |v, i| v.and_then(|sv| sv.get(i)))
    }
    fn gotten_mut(self, value: &mut Value) -> Option<&mut Self::Output> {
        self.iter().fold(Some(value), |v, i| v.and_then(|sv| sv.get_mut(i)))
    }
    fn indexed(self, value: &Value) -> &Self::Output {
        self.iter().fold(value, |v, i| &v[i])
    }
    fn indexed_mut(self, value: &mut Value) -> &mut Self::Output {
        self.iter().fold(value, |v, i| &mut v[i])
    }
}

impl JsonPath {
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }
    pub fn push(&mut self, indexer: JsonIndexer) {
        self.path.push(indexer)
    }
    pub fn pop(&mut self) -> Option<JsonIndexer> {
        self.path.pop()
    }
    pub fn iter(&self) -> impl Iterator<Item = &JsonIndexer> {
        self.path.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut JsonIndexer> {
        self.path.iter_mut()
    }
    pub fn last(&self) -> Option<&JsonIndexer> {
        self.path.last()
    }
    pub fn split_last(&self) -> Option<(&[JsonIndexer], &JsonIndexer)> {
        self.path.split_last().map(|(t, h)| (h, t))
    }
}

impl From<&[JsonIndexer]> for JsonPath {
    fn from(indexer: &[JsonIndexer]) -> Self {
        Self { path: indexer.into() }
    }
}
impl FromIterator<JsonIndexer> for JsonPath {
    fn from_iter<T: IntoIterator<Item = JsonIndexer>>(iter: T) -> Self {
        Self { path: iter.into_iter().collect() }
    }
}
impl IntoIterator for JsonPath {
    type Item = JsonIndexer;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.path.into_iter()
    }
}

impl std::fmt::Display for JsonPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self
            .iter()
            .map(|ji| match ji {
                JsonIndexer::ObjInd(s) => quote(s),
                JsonIndexer::ArrInd(i) => i.to_string(),
            })
            .join(">");
        write!(f, "{}", path)
    }
}

impl std::hash::Hash for JsonPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_access_by_path() {
        let json = r#"{ "key": [ 1, "two", { "foo": "bar" } ] }"#;
        let ast_root = Value::parse(json).unwrap();

        let path: JsonPath = vec![
            JsonIndexer::ObjInd("key".to_string()),
            JsonIndexer::ArrInd(2),
            JsonIndexer::ObjInd("foo".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(ast_root[&path], Value::String("bar".to_string()));
    }
}
