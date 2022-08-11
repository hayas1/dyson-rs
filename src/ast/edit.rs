use super::Value;

impl Value {
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

    pub fn update_with<F: FnOnce(&Value) -> Value>(&mut self, f: F) {
        std::mem::swap(self, &mut f(self))
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

        json["key"].update_with(|val| val.array().iter().map(|v| Value::from(v.integer() + 1)).collect());

        assert_eq!(json, Value::parse(r#"{"key": [1, 2, 3, 4], "foo": {"bar": "baz"}}"#).unwrap());
    }
}
