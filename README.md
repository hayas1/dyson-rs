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

// write json
json.dump("path/to/write.json").expect("failed to write json");
```