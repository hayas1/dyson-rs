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
pub enum DfsEvent<'a> {
    Visit(&'a Value),
    Leave(&'a Value),
    ForwardEdge(&'a Value, &'a Value),
    BackEdge(&'a Value, &'a Value),
}

impl Value {
    /// walk json [`Value`] with bfs order. if `f` return true continue walk, return false interrupt walk.
    /// if complete walk, this method return true, and not complete walk, this method return false.
    /// # examples
    /// ```
    /// use dyson::{Value, DfsEvent};
    /// let raw_json = r#"{ "key": [ 1, "two", 3, { "foo": { "bar": "baz" } } ] }"#;
    /// let json = Value::parse(raw_json).unwrap();
    ///
    /// let (mut depth, mut max_depth) = (0, 0);
    /// json.walk(|event| match event {
    ///     DfsEvent::Visit(_v) => true,
    ///     DfsEvent::Leave(_v) => true,
    ///     DfsEvent::ForwardEdge(_parent, _child) => {
    ///         depth = depth + 1;
    ///         max_depth = max_depth.max(depth);
    ///         true
    ///     }
    ///     DfsEvent::BackEdge(_child, _parent) => {
    ///         depth = depth - 1;
    ///         max_depth = max_depth.max(depth);
    ///         true
    ///     }
    /// });
    /// assert_eq!(max_depth, 4);
    /// ```
    pub fn walk<'a, F: FnMut(DfsEvent<'a>) -> bool>(&'a self, mut f: F) -> bool {
        // FIXME for return false, use proc_macro?
        let (mut stack, mut iter_stack) = (Vec::new(), Vec::new());
        match self {
            Value::Object(m) => {
                if !f(DfsEvent::Visit(self)) {
                    return false;
                }
                stack.push(self);
                iter_stack.push(ValueIterator::ObjectIterator(m.iter()));
            }
            Value::Array(v) => {
                if !f(DfsEvent::Visit(self)) {
                    return false;
                }
                stack.push(self);
                iter_stack.push(ValueIterator::ArrayIterator(v.iter()));
            }
            v => {
                if !f(DfsEvent::Visit(v)) {
                    return false;
                }
                if !f(DfsEvent::Leave(v)) {
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
                    if !f(DfsEvent::ForwardEdge(lis, next_value)) {
                        return false;
                    }
                    stack.push(next_value);
                    iter_stack.push(ValueIterator::ObjectIterator(m.iter()));
                    if !f(DfsEvent::Visit(next_value)) {
                        return false;
                    }
                }
                Some(Value::Array(v)) => {
                    let next_value = next.unwrap();
                    if !f(DfsEvent::ForwardEdge(lis, next_value)) {
                        return false;
                    }
                    stack.push(next_value);
                    iter_stack.push(ValueIterator::ArrayIterator(v.iter()));
                    if !f(DfsEvent::Visit(next_value)) {
                        return false;
                    }
                }
                Some(v) => {
                    if !f(DfsEvent::ForwardEdge(last, v)) {
                        return false;
                    }
                    if !f(DfsEvent::Visit(v)) {
                        return false;
                    }
                    if !f(DfsEvent::Leave(v)) {
                        return false;
                    }
                    if !f(DfsEvent::BackEdge(v, last)) {
                        return false;
                    }
                }
                None => {
                    iter_stack.pop();
                    if let Some(v) = stack.pop() {
                        if !f(DfsEvent::Leave(v)) {
                            return false;
                        }
                        let parent = stack.last().copied();
                        if parent.is_some() && !f(DfsEvent::BackEdge(v, parent.unwrap())) {
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
            DfsEvent::Visit(v) => assert_eq!(v, &Value::String("rust".into())) == (),
            DfsEvent::Leave(v) => assert_eq!(v, &Value::String("rust".into())) == (),
            DfsEvent::ForwardEdge(_, _) => unreachable!("one element json has no edge"),
            DfsEvent::BackEdge(_, _) => unreachable!("one element json has no edge"),
        }));

        assert!(!json.walk(|event| match event {
            DfsEvent::Visit(_) => false,
            DfsEvent::Leave(_) => unreachable!("when visit first node, return false"),
            DfsEvent::ForwardEdge(_, _) => unreachable!("one element json has no edge"),
            DfsEvent::BackEdge(_, _) => unreachable!("one element json has no edge"),
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
        assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json)));
        assert_eq!(iter.next(), Some(&DfsEvent::ForwardEdge(&json, &json["key"])));
        assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json["key"])));
        {
            assert_eq!(iter.next(), Some(&DfsEvent::ForwardEdge(&json["key"], &json["key"][0])));
            assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json["key"][0])));
            assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json["key"][0])));
            assert_eq!(iter.next(), Some(&DfsEvent::BackEdge(&json["key"][0], &json["key"])));

            assert_eq!(iter.next(), Some(&DfsEvent::ForwardEdge(&json["key"], &json["key"][1])));
            assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json["key"][1])));
            assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json["key"][1])));
            assert_eq!(iter.next(), Some(&DfsEvent::BackEdge(&json["key"][1], &json["key"])));

            assert_eq!(iter.next(), Some(&DfsEvent::ForwardEdge(&json["key"], &json["key"][2])));
            assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json["key"][2])));
            {
                assert_eq!(iter.next(), Some(&DfsEvent::ForwardEdge(&json["key"][2], &json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfsEvent::Visit(&json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json["key"][2]["foo"])));
                assert_eq!(iter.next(), Some(&DfsEvent::BackEdge(&json["key"][2]["foo"], &json["key"][2])));
            }
            assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json["key"][2])));
            assert_eq!(iter.next(), Some(&DfsEvent::BackEdge(&json["key"][2], &json["key"])));
        }
        assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json["key"])));
        assert_eq!(iter.next(), Some(&DfsEvent::BackEdge(&json["key"], &json)));

        assert_eq!(iter.next(), Some(&DfsEvent::Leave(&json)));
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
