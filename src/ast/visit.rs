use std::{collections::hash_map, slice};

use super::Value;

pub struct DfsVisitor<'a> {
    stack: Vec<ValueIterator<'a>>,
    first: Option<&'a Value>,
}
enum ValueIterator<'a> {
    ObjectIterator(hash_map::Iter<'a, String, Value>),
    ArrayIterator(slice::Iter<'a, Value>),
}
pub enum DfSEvent<'a> {
    Visit(&'a Value),
    Leave(&'a Value),
    ForwardEdge(&'a Value, &'a Value),
    BackEdge(&'a Value, &'a Value),
}

impl Value {
    fn walk<F: FnMut(DfSEvent) -> bool>(&self) {
        todo!();
    }

    /// get json visitor it will visit [`Value`] with bfs order.
    /// # examples
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{ "key": [ 1, "two", 3, { "foo": { "bar": "baz" } } ] }"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// let expected = vec![Value::Integer(1), Value::String("two".to_string()), Value::Integer(3), Value::String("baz".to_string())];
    /// for (visited, expected) in json.visitor().zip(expected) {
    ///     assert_eq!(visited, &expected);
    /// }
    /// ```
    pub fn visitor(&self) -> DfsVisitor {
        match self {
            Value::Object(m) => DfsVisitor { stack: vec![ValueIterator::ObjectIterator(m.iter())], first: None },
            Value::Array(v) => DfsVisitor { stack: vec![ValueIterator::ArrayIterator(v.iter())], first: None },
            v => DfsVisitor { stack: vec![], first: Some(v) },
        }
    }
}

impl<'a> Iterator for DfsVisitor<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first.is_some() {
            self.first.take()
        } else {
            while let Some(last) = self.stack.last_mut() {
                let next = match last {
                    ValueIterator::ObjectIterator(oi) => oi.next().map(|(_k, v)| v),
                    ValueIterator::ArrayIterator(ai) => ai.next(),
                };
                match next {
                    Some(Value::Object(m)) => self.stack.push(ValueIterator::ObjectIterator(m.iter())),
                    Some(Value::Array(v)) => self.stack.push(ValueIterator::ArrayIterator(v.iter())),
                    Some(v) => return Some(v),
                    None => {
                        self.stack.pop();
                    }
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;
    #[test]
    fn test_stringify_json() {
        #[derive(Hash, PartialEq, Eq, Debug)]
        enum Either {
            String(String),
            Integer(i64),
        }
        let json = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 1,"#,
            r#"    "keyword": ["rust", "json", "parser", 1, 2],"#,
            r#"    "dict": {"one": 1, "two": 2, "three": 3}"#,
            r#"}"#,
        ]
        .join("\n");
        let root = Value::parse(json).unwrap();
        let mut counter = HashMap::new();
        for visited in root.visitor() {
            match visited {
                Value::String(s) => *counter.entry(Either::String(s.clone())).or_insert(0) += 1,
                Value::Integer(i) => *counter.entry(Either::Integer(*i)).or_insert(0) += 1,
                _ => unreachable!("sample json consists of only String and Integer"),
            }
        }
        let expected: HashMap<_, _> = vec![
            (Either::String("rust".into()), 2),
            (Either::String("json".into()), 2),
            (Either::String("parser".into()), 1),
            (Either::Integer(1), 3),
            (Either::Integer(2), 2),
            (Either::Integer(3), 1),
        ]
        .into_iter()
        .collect();
        assert_eq!(counter, expected);
    }
}
