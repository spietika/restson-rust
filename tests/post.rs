extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,RestPath};

#[derive(Serialize,Deserialize)]
struct HttpBinPost {
    data: String,
}

#[derive(Deserialize)]
struct HttpBinPostResp {
    json: HttpBinPost,
    url: String,
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

#[test]
fn post_capture() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data = HttpBinPost { data: String::from("test data")};
    let resp: HttpBinPostResp = client.post_capture((), &data).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "http://httpbin.org/post");
}

#[test]
fn post_capture_query_params() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a","2"), ("b","abcd")];
    let data = HttpBinPost { data: String::from("test data")};
    let resp: HttpBinPostResp = client.post_capture_with((), &data, &params).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "http://httpbin.org/post?a=2&b=abcd");
}