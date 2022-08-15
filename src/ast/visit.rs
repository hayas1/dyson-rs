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
#[derive(Debug, PartialEq)]
pub enum DfSEvent<'a> {
    Visit(&'a Value),
    Leave(&'a Value),
    ForwardEdge(&'a Value, &'a Value),
    BackEdge(&'a Value, &'a Value),
}

impl Value {
    pub fn walk<'a, F: FnMut(DfSEvent<'a>) -> bool>(&'a self, mut f: F) -> bool {
        // FIXME for return false, use proc_macro?
        let (mut stack, mut iter_stack) = (Vec::new(), Vec::new());
        match self {
            Value::Object(m) => {
                if !f(DfSEvent::Visit(self)) {
                    return false;
                }
                stack.push(self);
                iter_stack.push(ValueIterator::ObjectIterator(m.iter()));
            }
            Value::Array(v) => {
                if !f(DfSEvent::Visit(self)) {
                    return false;
                }
                stack.push(self);
                iter_stack.push(ValueIterator::ArrayIterator(v.iter()));
            }
            v => {
                if !f(DfSEvent::Visit(v)) {
                    return false;
                }
                if !f(DfSEvent::Leave(v)) {
                    return false;
                }
            }
        }
        while let (Some(last), Some(last_iter)) = (stack.last(), iter_stack.last_mut()) {
            let (lis, next) = match (last, last_iter) {
                (lis, ValueIterator::ObjectIterator(oi)) => (lis, oi.next().map(|(_k, v)| v)),
                (lis, ValueIterator::ArrayIterator(ai)) => (lis, ai.next()),
            };
            match next {
                Some(Value::Object(m)) => {
                    let next_value = next.unwrap();
                    if !f(DfSEvent::ForwardEdge(lis, next_value)) {
                        return false;
                    }
                    stack.push(next_value);
                    iter_stack.push(ValueIterator::ObjectIterator(m.iter()));
                    if !f(DfSEvent::Visit(next_value)) {
                        return false;
                    }
                }
                Some(Value::Array(v)) => {
                    let next_value = next.unwrap();
                    if !f(DfSEvent::ForwardEdge(lis, next_value)) {
                        return false;
                    }
                    stack.push(next_value);
                    iter_stack.push(ValueIterator::ArrayIterator(v.iter()));
                    if !f(DfSEvent::Visit(next_value)) {
                        return false;
                    }
                }
                Some(v) => {
                    if !f(DfSEvent::ForwardEdge(last, v)) {
                        return false;
                    }
                    if !f(DfSEvent::Visit(v)) {
                        return false;
                    }
                    if !f(DfSEvent::Leave(v)) {
                        return false;
                    }
                    if !f(DfSEvent::BackEdge(v, last)) {
                        return false;
                    }
                }
                None => {
                    iter_stack.pop();
                    if let Some(v) = stack.pop() {
                        if !f(DfSEvent::Leave(v)) {
                            return false;
                        }
                        let parent = stack.last().copied();
                        if parent.is_some() && !f(DfSEvent::BackEdge(v, parent.unwrap())) {
                            return false;
                        }
                    }
                }
            }
        }
        true
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
    #[allow(clippy::unit_cmp)]
    fn test_walk1value_json() {
        let raw_json = "\"rust\"";
        let json = Value::parse(raw_json).unwrap();
        assert!(json.walk(|event| match event {
            DfSEvent::Visit(v) => assert_eq!(v, &Value::String("rust".into())) == (),
            DfSEvent::Leave(v) => assert_eq!(v, &Value::String("rust".into())) == (),
            DfSEvent::ForwardEdge(_, _) => unreachable!("one element json has no edge"),
            DfSEvent::BackEdge(_, _) => unreachable!("one element json has no edge"),
        }));

        assert!(!json.walk(|event| match event {
            DfSEvent::Visit(_) => false,
            DfSEvent::Leave(_) => unreachable!("when visit first node, return false"),
            DfSEvent::ForwardEdge(_, _) => unreachable!("one element json has no edge"),
            DfSEvent::BackEdge(_, _) => unreachable!("one element json has no edge"),
        }));
    }

    #[test]
    #[allow(clippy::unit_cmp)]
    fn test_walk_json() {
        let raw_json = r#"{ "key": [ 1, "two", { "foo": "bar" } ] }"#;
        let json = Value::parse(raw_json).unwrap();
        let mut events = Vec::new();
        assert!(json.walk(|event| events.push(event) == ()));
        let mut iter = events.iter();
        assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json)));
        assert_eq!(iter.next(), Some(&DfSEvent::ForwardEdge(&json, &json["key"])));
        assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json["key"])));
        {
            assert_eq!(iter.next(), Some(&DfSEvent::ForwardEdge(&json["key"], &json["key"][0])));
            assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json["key"][0])));
            assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json["key"][0])));
            assert_eq!(iter.next(), Some(&DfSEvent::BackEdge(&json["key"][0], &json["key"])));

            assert_eq!(iter.next(), Some(&DfSEvent::ForwardEdge(&json["key"], &json["key"][1])));
            assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json["key"][1])));
            assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json["key"][1])));
            assert_eq!(iter.next(), Some(&DfSEvent::BackEdge(&json["key"][1], &json["key"])));

            assert_eq!(iter.next(), Some(&DfSEvent::ForwardEdge(&json["key"], &json["key"][2])));
            assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json["key"][2])));
            {
                assert_eq!(iter.next(), Some(&DfSEvent::ForwardEdge(&json["key"][2], &json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfSEvent::Visit(&json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfSEvent::BackEdge(&json["key"][2]["foo"], &json["key"][2])));
            }
            assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json["key"][2])));
            assert_eq!(iter.next(), Some(&DfSEvent::BackEdge(&json["key"][2], &json["key"])));
        }
        assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json["key"])));
        assert_eq!(iter.next(), Some(&DfSEvent::BackEdge(&json["key"], &json)));

        assert_eq!(iter.next(), Some(&DfSEvent::Leave(&json)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_visit_json() {
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
