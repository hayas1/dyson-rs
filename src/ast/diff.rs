use super::{index::JsonIndexer, index_path::JsonPath, Value};
use itertools::Itertools;

/// compare `a` and `b` that are expected same structure. this method's complexity is **O(max{|a|, |b|})**.
/// # panics
/// if 'a' and 'b' do not have same structure.
pub fn diff_value(a: &Value, b: &Value) -> Vec<(JsonPath, JsonPath)> {
    fn diff_value_recursive(
        (a, b): (&Value, &Value),
        (path_a, path_b): (&mut JsonPath, &mut JsonPath),
        differences: &mut Vec<(JsonPath, JsonPath)>,
    ) {
        match (a, b) {
            (Value::Object(ma), Value::Object(mb)) => {
                let (mai, mbi) = (ma.iter().sorted_by_key(|e| e.0), mb.iter().sorted_by_key(|e| e.0));
                for ((mak, mav), (mbk, mbv)) in itertools::zip_eq(mai, mbi) {
                    path_a.push(JsonIndexer::ObjInd(mak.to_string()));
                    path_b.push(JsonIndexer::ObjInd(mbk.to_string()));
                    if mak == mbk {
                        diff_value_recursive((mav, mbv), (path_a, path_b), differences);
                    } else {
                        differences.push((path_a.clone(), path_b.clone()));
                    }
                    path_b.pop();
                    path_a.pop();
                }
            }
            (Value::Array(va), Value::Array(vb)) => {
                for (i, (vav, vbv)) in itertools::zip_eq(va, vb).enumerate() {
                    path_a.push(JsonIndexer::ArrInd(i));
                    path_b.push(JsonIndexer::ArrInd(i));
                    diff_value_recursive((vav, vbv), (path_a, path_b), differences);
                    path_b.pop();
                    path_a.pop();
                }
            }
            (av, bv) => {
                if av != bv {
                    differences.push((path_a.clone(), path_b.clone()));
                }
            }
        }
    }
    let mut differences = Vec::new();
    diff_value_recursive((a, b), (&mut JsonPath::new(), &mut JsonPath::new()), &mut differences);
    differences
}

/// compare `a` and `b` that are expected same structure. with human friendly message.
/// this method's complexity is **O(max{|a|, |b|})**.
/// # panics
/// if 'a' and 'b' do not have same structure.
pub fn diff_value_detail(a: &Value, b: &Value) -> Vec<String> {
    let mut result = Vec::new();
    let path = diff_value(a, b);
    for (pa, pb) in path {
        if pa.last() == pb.last() {
            result.push(format!("{}: different value {} and {}", pa, a[&pa], b[&pb]));
        } else {
            let ((prefix, pal), pbl) =
                (pa.split_last().map_or_else(|| (JsonPath::new(), None), |(h, t)| (h, Some(t))), pb.last());
            match (pal, pbl) {
                (Some(pal), Some(pbl)) => {
                    result.push(format!("{}: different key {:?} and {:?}", prefix, pal, pbl));
                }
                _ => unreachable!("above function ensure that pa and pb have same length"),
            }
        }
    }
    result
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
                (
                    JsonPath::from(&vec![JsonIndexer::ObjInd("keyword".to_string()), JsonIndexer::ArrInd(2)][..]),
                    JsonPath::from(&vec![JsonIndexer::ObjInd("keyword".to_string()), JsonIndexer::ArrInd(2)][..]),
                ),
                (
                    JsonPath::from(&vec![JsonIndexer::ObjInd("language".to_string())][..]),
                    JsonPath::from(&vec![JsonIndexer::ObjInd("language".to_string())][..]),
                ),
            ]
            .iter()
            .collect::<HashSet<_>>()
        );
        for (path1, path2) in diff_path {
            assert_ne!(ast_root1[&path1], ast_root2[&path2]);
        }
    }

    #[test]
    #[allow(clippy::single_char_pattern)]
    fn test_diff_value_detail_json() {
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
            r#"    "language": "rust","#,
            r#"    "notation": "json","#,
            r#"    "version": 0.1,"#,
            r#"    "keyword": ["ruby", "json", "parser", 1, 2, 3]"#,
            r#"}"#,
        ];
        let ast_root1 = Value::parse(json1.into_iter().collect::<String>()).unwrap();
        let ast_root2 = Value::parse(json2.into_iter().collect::<String>()).unwrap();

        let diff = diff_value_detail(&ast_root1, &ast_root2);
        assert!(diff[0].contains("keyword"));
        assert!(diff[0].contains("0"));
        assert!(diff[0].contains("rust"));
        assert!(diff[0].contains("ruby"));
    }
}
