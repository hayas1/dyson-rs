# dyson
dynamic json parser (more, see [document](https://hayas1.github.io/dyson/dyson/)).

# usage
## lib
# usage
in `Cargo.toml`.
```toml
[dependencies]
    dyson = { git = "https://github.com/hayas1/dyson" }
```

## cli
### install
```sh
$ cargo install --git https://github.com/hayas1/dyson
```
### uninstall
```sh
$ cargo uninstall dyson
```
# examples
## lib
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

## cli
### command
#### help
```sh
$ dyson --help
dyson

USAGE:
    dyson <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    compare    compare two json
    format     format json
    help       Print this message or the help of the given subcommand(s)
```
#### format
format output will be random...
```sh
$ dyson format path/to/read.json
{
    "version": 0.1,
    "language": "rust",
    "notation": "json",
    "keyword": [
        "rust",
        "json",
        "parser"
    ]
}
```

```sh
$ cat path/to/read.json | dyson format
{
    "notation": "json",
    "version": 0.1,
    "language": "rust",
    "keyword": [
        "rust",
        "json",
        "parser"
    ]
}
```

#### compare
too simple compare...
```sh
$ dyson compare path/to/read.json path/to/write.json
false
```

```sh
$ cat path/to/read.json | dyson compare path/to/read.json
true
```
