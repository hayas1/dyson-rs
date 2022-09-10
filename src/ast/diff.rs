use super::{
    index::{JsonIndexer, JsonPath},
    Value,
};
use itertools::Itertools;

/// **O(max{|a|, |b|})**, compare `a` and `b` that are expected same structure.
/// returned [`JsonPath`] is based on `a`'s structure.
/// # panics
/// if 'a' and 'b' do not have same structure.
pub fn diff_value(a: &Value, b: &Value) -> Vec<JsonPath> {
    fn diff_value_recursive(a: &Value, b: &Value, path: &mut JsonPath, differences: &mut Vec<JsonPath>) {
        match (a, b) {
            (Value::Object(ma), Value::Object(mb)) => {
                let (mai, mbi) = (ma.iter().sorted_by_key(|e| e.0), mb.iter().sorted_by_key(|e| e.0));
                for ((mak, mav), (mbk, mbv)) in itertools::zip_eq(mai, mbi) {
                    path.push(JsonIndexer::ObjInd(mak.to_string()));
                    if mak == mbk {
                        diff_value_recursive(mav, mbv, path, differences);
                    } else {
                        differences.push(path.clone())
                    }
                    path.pop();
                }
            }
            (Value::Array(va), Value::Array(vb)) => {
                for (i, (vav, vbv)) in itertools::zip_eq(va, vb).enumerate() {
                    path.push(JsonIndexer::ArrInd(i));
                    diff_value_recursive(vav, vbv, path, differences);
                    path.pop();
                }
            }
            (av, bv) => {
                if av != bv {
                    differences.push(path.clone())
                }
            }
        }
    }
    let mut differences = Vec::new();
    diff_value_recursive(a, b, &mut Vec::new(), &mut differences);
    differences
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_diff_value_json() {
        let json1 = [
            r#"{"#,
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "parser", 1, 2, 3]"#,
            r#"}"#,
        ];
        let json2 = [
            r#"{"#,
            r#"    "language": "ruby","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["rust", "json", "tokenizer", 1, 2, 3]"#,
            r#"}"#,
        ];
        let ast_root1 = Value::parse(json1.into_iter().collect::<String>()).unwrap();
        let ast_root2 = Value::parse(json2.into_iter().collect::<String>()).unwrap();

        let diff_path = diff_value(&ast_root1, &ast_root2);
        assert_eq!(
            diff_path.iter().collect::<HashSet<_>>(),
            vec![
                vec![JsonIndexer::ObjInd("keyword".to_string()), JsonIndexer::ArrInd(2)],
                vec![JsonIndexer::ObjInd("language".to_string())]
            ]
            .iter()
            .collect::<HashSet<_>>()
        );
        for path in diff_path {
            assert_ne!(ast_root1[&path], ast_root2[&path]);
        }
    }
}
