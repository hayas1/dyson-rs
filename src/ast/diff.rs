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
