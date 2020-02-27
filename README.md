
[![crates.io](https://img.shields.io/crates/v/restson.svg)](https://crates.io/crates/restson) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://raw.githubusercontent.com/spietika/restson-rust/master/LICENSE) [![Docs: latest](https://img.shields.io/badge/Docs-latest-green.svg)](https://docs.rs/restson/)

# Restson Rust

Easy-to-use REST client for Rust programming language that provides automatic serialization and deserialization from Rust structs. The library is implemented using [Hyper](https://github.com/hyperium/hyper) and [Serde JSON](https://github.com/serde-rs/json).

## Getting started

Add the following lines to your project `Cargo.toml` file:

```toml
[dependencies]
restson = "^0.6"
serde = "^1.0"
serde_derive = "^1.0"
```
This adds dependencies for the Restson library and also for Serde which is needed to derive `Serialize` and `Deserialize` for user defined data structures.

### Data structures

Next, the data structures for the REST interface should be defined. The struct fields need to match with the API JSON fields. The whole JSON does not need to be defined, the struct can also contain a subset of the fields. Structs that are used with `GET` should derive `Deserialize` and structs that are used with `POST` should derive `Serialize`.

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

Each type that is used with `get`/`post` needs to implement `RestPath` trait. The trait can be implemented multiple times with different generic parameters for the same type as shown below. The `get_path` can also return error to indicate that the parameters were not valid. This error is propagated directly to the client caller.

```rust
// plain API call without parameters
impl RestPath<()> for HttpBinAnything {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("anything")) }
}

// API call with one u32 parameter (e.g. "http://httpbin.org/anything/1234")
impl RestPath<u32> for HttpBinAnything {
    fn get_path(param: u32) -> Result<String,Error> { Ok(format!("anything/{}", param)) }
}
```

### Requests

To run requests the client instance needs to be created first. The base URL of the resource is given as parameter:
```rust
let mut client = RestClient::new("http://httpbin.org").unwrap();
```

This creates a client instance with default configuration. To configure the client, it is created with a `Builder`

```rust
let mut client = RestClient::builder().dns_workers(1)
        .build("http://httpbin.org").unwrap();
```

**GET**

The following snippet shows an example `GET` request:
```rust
// Gets https://httpbin.org/anything/1234 and deserializes the JSON to data variable
let data: HttpBinAnything = client.get(1234).unwrap();
```
The `get` and `post` functions call the `get_path` function automatically from `RestPath` based on the parameter type to construct the URL. If the compiler is able to infer the parameter and return value types from the context (as shown above), they do not need to be annotated. The call above is equivalent with:

```rust
let data = client.get::<u32, HttpBinAnything>(1234).unwrap();
```
Restson also provides `get_with` function which is similar to the basic `get` but it also accepts additional query parameters that are added to the request URL.
```rust
// Gets http://httpbin.org/anything/1234?a=2&b=abcd
let query = vec![("a","2"), ("b","abcd")];
let data: HttpBinAnything = client.get_with(1234, &query).unwrap();
```
Both GET interfaces return `Result<T, Error>` where T is the target type in which the returned JSON is deserialized to.

**POST**

The following snippets show an example `POST` request:
```rust
#[derive(Serialize)]
struct HttpBinPost {
    data: String,
}

impl RestPath<()> for HttpBinPost {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("post")) }
}
```
```rust
let data = HttpBinPost { data: String::from("test data")};
// Posts data to http://httpbin.org/post
client.post((), &data).unwrap();
```
In addition to the basic `post` interface, it is also possible to provide query parameters with `post_with` function. Also, `post_capture` and `post_capture_with` interfaces allow to capture and deserialize the message body returned by the server in the POST request.

**PUT**

HTTP PUT requests are also supported and the interface is similar to POST interface: `put`, `put_with`, `put_capture` and `put_capture_with` functions are available.

**PATCH**

HTTP PATCH requests are also supported and the interface is similar to POST and PUT interface: `patch` and `patch_with` functions are available.

**DELETE**

Restson supports HTTP DELETE requests to API paths. Normally DELETE request is sent to API URL without message body. However, if message body or query parameters are needed, `delete_with` can be used. Moreover, the response status code from server is checked, but the response body is not captured.

Similarly with other requests, the path is obtained from `RestPath` trait.

```rust
struct HttpBinDelete {
}

impl RestPath<()> for HttpBinDelete {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("delete")) }
}
```

The `delete` function does not return any data (only possible error) so the type needs to be annotated.

```rust
// DELETE request to http://httpbin.org/delete
let mut client = RestClient::new("http://httpbin.org").unwrap();
client.delete::<(), HttpBinDelete>(()).unwrap();
```

### JSON with array root element

In all of the examples above the JSON structure consists of key-value pairs that can be represented with Rust structs. However, it is also possible that valid JSON has array root element without a key. For example, the following is valid JSON.

```json
["a","b","c"]
```

It is possible to work with APIs returning arrays in Restson. However instead of a struct, the user type needs to be a container. `Vec<String>` in this case. The type also needs to implement the `RestPath` trait as explained before, and easiest way to do so is to wrap the container in a `struct`.

```rust
#[derive(Serialize,Deserialize,Debug,Clone)]
struct Products ( pub Vec<Product> );

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Product {
    pub name: String,
    //...
}

impl RestPath<()> for Products {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("/api/objects/products"))}
}

pub fn products(&self) -> Vec<Product> {
    let mut client = RestClient::new("http://localhost:8080").unwrap();
    client.get::<_, Products>(()).unwrap().0
}
```

### Logging
The library uses the `log` crate to provide debug and trace logs. These logs allow to easily see both outgoing requests as well as incoming responses from the server. See the [log crate documentation](https://docs.rs/log/*/log/) for details.

### Examples
For more examples see *tests* directory. 

## License

The library is released under the MIT license. See [LICENSE](https://raw.githubusercontent.com/spietika/restson-rust/master/LICENSE) for details.
