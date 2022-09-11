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
    pub fn strip_prefix(&self, prefix: &Self) -> Option<Self> {
        self.path.strip_prefix(&prefix.path[..]).map(|indexers| indexers.into())
    }
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.path.starts_with(&prefix.path)
    }
    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.path.strip_suffix(&suffix.path[..]).map(|indexers| indexers.into())
    }
    pub fn ends_with(&self, suffix: &Self) -> bool {
        self.path.ends_with(&suffix.path)
    }
}
impl JsonPath {
    /// get depth.
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// get lowest common ancestor. this method's complexity is **O(`lca.depth()`)**.
    pub fn lca(a: &Self, b: &Self) -> Self {
        let mut result = Self::new();
        for (ai, _bi) in itertools::zip(a, b).take_while(|(ai, bi)| ai == bi) {
            result.push(ai.clone());
        }
        result
    }

    /// get parent.
    pub fn parent(&self) -> Option<Self> {
        self.split_last().map(|(h, _t)| h)
    }

    /// get ancestors. this method's complexity is **O(`depth`^2)**.
    pub fn ancestors(&self) -> impl Iterator<Item = Self> {
        let origin = self.clone();
        (0..=self.depth()).scan(origin, |a, _| {
            let ret = Some(a.clone());
            a.pop();
            ret
        })
    }

    /// append `path` to back of `self`.
    pub fn join(&self, path: &Self) -> Self {
        self.iter().chain(path).cloned().collect()
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

impl Extend<JsonIndexer> for JsonPath {
    fn extend<T: IntoIterator<Item = JsonIndexer>>(&mut self, iter: T) {
        self.path.extend(iter.into_iter())
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
impl Into<Vec<JsonIndexer>> for JsonPath {
    fn into(self) -> Vec<JsonIndexer> {
        self.path
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

    #[test]
    fn test_vec_like_interface() {
        let json = r#"{ "key": [ 1, "two", { "foo": "bar" } ] }"#;
        let ast_root = Value::parse(json).unwrap();

        let mut path: JsonPath = vec![JsonIndexer::ObjInd("key".to_string())].into_iter().collect();
        assert_eq!(ast_root[&path], Value::parse(r#"[ 1, "two", { "foo": "bar" } ]"#).unwrap());
        path.push(JsonIndexer::ArrInd(0));
        assert_eq!(ast_root[&path], Value::parse(r#"1"#).unwrap());
        path.pop();
        path.push(JsonIndexer::ArrInd(2));
        assert_eq!(ast_root[&path], Value::parse(r#"{ "foo": "bar" }"#).unwrap());

        assert!(path.starts_with(&vec![JsonIndexer::ObjInd("key".to_string())].into_iter().collect()));
        assert!(path
            .starts_with(&vec![JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(2)].into_iter().collect()));
        assert!(path.ends_with(&vec![JsonIndexer::ArrInd(2)].into_iter().collect()));
        assert!(
            path.ends_with(&vec![JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(2)].into_iter().collect())
        );
    }

    #[test]
    fn test_path_like_interface() {
        let json = r#"{ "key": [ 1, "two", { "foo": "bar" } ] }"#;
        let ast_root = Value::parse(json).unwrap();

        let path1: JsonPath =
            vec![JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(0)].into_iter().collect();
        let path2: JsonPath = vec![
            JsonIndexer::ObjInd("key".to_string()),
            JsonIndexer::ArrInd(2),
            JsonIndexer::ObjInd("foo".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!((path1.depth(), path2.depth()), (2, 3));
        assert_eq!(JsonPath::lca(&path1, &path2), vec![JsonIndexer::ObjInd("key".to_string())].into_iter().collect());
        assert_eq!(
            ast_root[&JsonPath::lca(&path1, &path2)],
            Value::parse(r#"[ 1, "two", { "foo": "bar" } ]"#).unwrap()
        );

        assert_eq!(path1.parent(), Some(vec![JsonIndexer::ObjInd("key".to_string())].into_iter().collect()));
        assert!(path1
            .ancestors()
            .zip(vec![
                vec![JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(0)].into_iter().collect(),
                vec![JsonIndexer::ObjInd("key".to_string())].into_iter().collect(),
                vec![].into_iter().collect(),
            ])
            .all(|(r, e)| r == e));

        let pa = JsonPath::from(&vec![JsonIndexer::ObjInd("key".to_string())][..]);
        let pb = JsonPath::from(&vec![JsonIndexer::ArrInd(2)][..]);
        assert_eq!(pa.join(&pb), JsonPath::from(&[JsonIndexer::ObjInd("key".to_string()), JsonIndexer::ArrInd(2)][..]));
        assert_eq!(ast_root[&pa.join(&pb)], Value::parse(r#"{ "foo": "bar" }"#).unwrap());
    }
}
