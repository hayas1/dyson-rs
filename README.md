# dyson
dynamic json parser (more, see [document](https://hayas1.github.io/dyson/dyson/)).

# usage
TODO

# examples
```rust
// `path/to/read.json`
// {
//     "language": "rust",
//     "notation": "json",
//     "version": 0.1,
//     "keyword": ["rust", "json", "parser"]
// }

use dyson::{Value, Ranger};
// read json
let json = Value::load("path/to/read.json").expect("invalid path or json structure");

// access json
if let Value::String(language) = &json["language"] {
    println!("{}", language) // rust
}
println!("{}", json["version"].float()); // 0.1
println!("{:?}", &json["keyword"][Ranger(1..)]); // [Value::String("json"), Value::String("parser")]
println!("{:?}", json.get("foo")); // None

//  edit json
let mut json = json;
json["language"] = "ruby".into();
println!("{}", json["language"].string()); // ruby
json["version"].swap(&mut 0.2.into());
println!("{}", json["version"].float()); // 0.2
json["keyword"].update_with(|v| v.iter().map(|k| Value::from(k.string().to_uppercase())).collect());
println!("{:?}", json["keyword"].array()); // ["RUST", "JSON", "PARSER"]

// write json
json.dump("path/to/write.json").expect("failed to write json");
// {
//     "version": 0.2,
//     "notation": "json",
//     "language": "ruby",
//     "keyword": [
//         "RUST",
//         "JSON",
//         "PARSER"
//     ]
// }
```