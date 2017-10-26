extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,RestPath};

#[derive(Serialize)]
struct HttpBinPost {
    data: String,
}

impl RestPath<()> for HttpBinPost {
    fn get_path(_: ()) -> String { String::from("post") }
}

#[test]
fn basic_post() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data = HttpBinPost { data: String::from("test data")};
    client.post((), &data).unwrap();
}

#[test]
fn post_query_params() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a","2"), ("b","abcd")];
    let data = HttpBinPost { data: String::from("test data")};
    client.post_with((), &data, &params).unwrap();
}