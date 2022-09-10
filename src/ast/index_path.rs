use super::{
    index::{JsonIndex, JsonIndexer},
    quote, Value,
};
use itertools::Itertools;
use std::iter::FromIterator;

/// [`JsonPath`] is used for accessing [`Value`]. see [`Value::get`] also.
/// # examples
/// ```
/// use dyson::{JsonIndexer, JsonPath, Value};
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
    pub fn get(&self, index: usize) -> Option<&JsonIndexer> {
        self.path.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut JsonIndexer> {
        self.path.get_mut(index)
    }
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, JsonIndexer> {
        self.path.iter()
    }
    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, JsonIndexer> {
        self.path.iter_mut()
    }
    pub fn last(&self) -> Option<&JsonIndexer> {
        self.path.last()
    }
    pub fn split_last(&self) -> Option<(JsonPath, &JsonIndexer)> {
        self.path.split_last().map(|(t, h)| (h.into(), t))
    }
}
impl JsonPath {
    /// get lowest common ancestor
    pub fn lca(a: &Self, b: &Self) -> Self {
        let mut result = Self::new();
        for (ai, _bi) in itertools::zip(a, b).take_while(|(ai, bi)| ai == bi) {
            result.push(ai.clone());
        }
        result
    }
}

impl<'a> std::ops::Index<usize> for JsonPath {
    type Output = JsonIndexer;
    fn index(&self, index: usize) -> &Self::Output {
        &self.path[index]
    }
}
impl<'a> std::ops::IndexMut<usize> for JsonPath {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.path[index]
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
impl<'a> IntoIterator for &'a JsonPath {
    type Item = &'a JsonIndexer;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        // FIXME why cannot compile?
        // type IntoIter = std::slice::Iter<'a, Self::Item>;
        // (&self.path).into_iter()
        (&self.path).into_iter().map(|x| x).collect_vec().into_iter()
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
