use super::Value;

impl Value {
    /// swap self and given value.
    /// # examples
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{"foo": [1, "two", 3], "bar": 4}"#;
    /// let mut json = Value::parse(raw_json).unwrap();
    ///
    /// let mut bar = json["bar"].swap(&mut ().into());
    /// assert_eq!(json, Value::parse(r#"{"foo": [1, "two", 3], "bar": null}"#).unwrap());
    ///
    /// let mut foo = json["foo"].swap(&mut bar);
    /// assert_eq!(json, Value::parse(r#"{"foo": 4, "bar": null}"#).unwrap());
    ///
    /// let null = json["bar"].swap(&mut foo);
    /// assert_eq!(json, Value::parse(r#"{"foo": 4, "bar": [1, "two", 3]}"#).unwrap());
    /// assert_eq!(null, Value::Null);
    /// ```
    pub fn swap(&mut self, value: &mut Value) -> Value {
        std::mem::swap(self, value);
        value.to_owned()
    }

    // TODO Sized dyn is impossible...?
    // pub fn update<F: Fn(&dyn Into<Value>) -> &dyn Into<Value>>(self, f: F) -> Value {
    //     let data: dyn Into<Value> = match self {
    //         Value::Object(m) => m,
    //         Value::Array(v) => v,
    //         Value::Bool(b) => b,
    //         Value::Null => &(),
    //         Value::String(s) => s,
    //         Value::Integer(i) => i,
    //         Value::Float(f) => f,
    //     };
    //     let mut prev = f(&data).into_value();
    //     std::mem::swap(&mut self, &mut prev);
    //     prev
    // }

    /// update value with previous value.
    /// # examples
    /// ```
    /// use dyson::Value;
    /// let raw_json = r#"{"foo": [1, "2", 3, "4", 5], "bar": 6}"#;
    /// let mut json = Value::parse(raw_json).unwrap();
    ///
    /// json["bar"].update_with(|v| (v.integer() * v.integer()).into());
    /// assert_eq!(json["bar"], 36.into());
    ///
    /// json["foo"].update_with(|v| {
    ///     v.iter().map( |e| {
    ///         Value::from(match e {
    ///             Value::String(s) => s.parse().unwrap(),
    ///             Value::Integer(i) => i * i,
    ///             _ => 0,
    ///         })
    ///     }).collect()
    /// });
    /// assert_eq!(json["foo"], vec![1.into(), 2.into(), 9.into(), 4.into(), 25.into()].into());
    /// assert_eq!(json, Value::parse(r#"{"foo": [1, 2, 9, 4, 25], "bar": 36}"#).unwrap())
    /// ```
    pub fn update_with<F: FnOnce(&Value) -> Value>(&mut self, f: F) -> Value {
        let mut prev = f(self);
        std::mem::swap(self, &mut prev);
        prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_node() {
        let raw = r#"{"key": ["zero", 1, "two", 3, {"foo": {"bar": "baz"}}]}"#;
        let mut json = Value::parse(raw).unwrap();

        json["key"][0] = 0.into();
        assert_eq!(json, Value::parse(r#"{"key": [0, 1, "two", 3, {"foo": {"bar": "baz"}}]}"#).unwrap());

        json["key"] = ().into();
        assert_eq!(json, Value::parse(r#"{"key": null}"#).unwrap());
    }

    #[test]
    fn test_swap_ast_node() {
        let raw = r#"{"key": ["zero", 1, "two", 3, {"foo": {"bar": "baz"}}]}"#;
        let mut json = Value::parse(raw).unwrap();

        let zero = json["key"][0].swap(&mut 0.into());
        let two = json["key"][2].swap(&mut 2.into());
        assert_eq!(zero, "zero".into());
        assert_eq!(two, "two".into());

        assert_eq!(json, Value::parse(r#"{"key": [0, 1, 2, 3, {"foo": {"bar": "baz"}}]}"#).unwrap());
    }

    #[test]
    fn test_update_ast_node() {
        let raw = r#"{"key": [0, 1, 2, 3], "foo": {"bar": "baz"}}"#;
        let mut json = Value::parse(raw).unwrap();

        json["key"].update_with(|val| val.iter().map(|v| Value::from(v.integer() + 1)).collect());

        assert_eq!(json, Value::parse(r#"{"key": [1, 2, 3, 4], "foo": {"bar": "baz"}}"#).unwrap());
    }

    #[test]
    fn test_insertion_order() {
        let raw = r#"{"foo": "hoge", "bar": "fuga", "baz": "piyo"}"#;
        let mut json = Value::parse(raw).unwrap();

        json.update_with(|val| {
            let mut cloned = val.object().clone();
            cloned.remove("bar");
            cloned.insert("one".to_string(), Value::from(1));
            cloned.insert("baz".to_string(), Value::from("piyo"));
            Value::from(cloned)
        });

        assert_eq!(json.to_string(), r#"{"foo":"hoge","one":1,"baz":"piyo"}"#)
    }
}
