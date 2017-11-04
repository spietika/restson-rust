# Restson Rust

Easy-to-use REST client for Rust programming language that provides automatic serialization and deserialization from Rust structs. The library is implemented using [Hyper](https://github.com/hyperium/hyper) and [Serde JSON](https://github.com/serde-rs/json).

## Getting started

Add the following lines to your project `Cargo.toml` file:

```toml
[dependencies]
restson = { git = "https://github.com/spietika/restson-rust"}
serde = "^1.0"
serde_derive = "^1.0"
```
This adds dependencies for the Restson library and also for Serde which is needed to derive `Serialize` and `Deserialize` for user defined data structures.

### Data structures

Next, the data structures for the REST interface should be defined. The struct fields need to match with the API JSON fields. The whole JSON does not need to be defined, the struct can also contain only subset of the fields. Structs that are used with `GET` should derive `Deserialize` and structs that are used with `POST` should derive `Serialize`.

Example JSON (subset of http://httpbin.org/anything response):
```json
{
  "method": "GET", 
  "origin": "1.2.3.4", 
  "url": "https://httpbin.org/anything"
}
```
Corresponding Rust struct:
```rust
#[macro_use]
extern crate serde_derive;

#[derive(Serialize,Deserialize)]
struct HttpBinAnything {
    method: String,
    url: String,
}
```

These definitions allow to automatically serialize/deserialize the data structures to/from JSON when requests are processed. For more complex scenarios, see the Serde [examples](https://serde.rs/examples.html).

### Rest paths

In Restson library the API resource paths are associated with types. That is, the URL is constructed automatically and not given as parameter to requests. This allows to easily parametrize the paths without manual URL processing and reduces URL literals in the code.

Each type that is used with `get`/`post` needs to implement `RestPath` trait. The trait can be implemented multiple times with different generic parameters for the same type as shown below.

```rust
// plain API call without parameters
impl RestPath<()> for HttpBinAnything {
    fn get_path(_: ()) -> String { String::from("anything") }
}

// API call with one u32 parameter (e.g. "http://httpbin.org/anything/1234")
impl RestPath<u32> for HttpBinAnything {
    fn get_path(param: u32) -> String { format!("anything/{}", param) }
}
```

### Requests

To run requests the client instance needs to be created first. The base URL of the resource is given as parameter:
```rust
let mut client = RestClient::new("http://httpbin.org").unwrap();
```

Example `GET` request:
```rust
// Gets https://httpbin.org/anything/1234 and deserializes the JSON to data variable
let data: HttpBinAnything = client.get(1234).unwrap();
```
The `get` and `post` functions call the `get_path` function automatically from `RestPath` based on the parameter type to construct the URL. If the compiler is able to infer the parameter and return value types from the context (as shown above), they do not need to be annotated. The call above is equivalent with:

```rust
let data = client.get::<u32, HttpBinAnything>(1234).unwrap();
```
### Examples
For more examples see *tests* directory. 

## License

The library is released under the MIT license. See [LICENSE](https://raw.githubusercontent.com/spietika/restson-rust/master/LICENSE) for details.
