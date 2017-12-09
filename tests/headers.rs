extern crate restson;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,RestPath,Error};
use hyper::header::{UserAgent};

#[derive(Deserialize)]
struct HttpBinAnything {
    headers: TestHeaders,
}

#[derive(Deserialize)]
struct TestHeaders {
    #[serde(default)]
    #[serde(rename = "User-Agent")]
    user_agent: String,
}

impl RestPath<()> for HttpBinAnything {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("anything")) }
}

#[test]
fn headers_raw() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_header_raw("User-Agent", "hyper/0.11.x");

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.headers.user_agent, "hyper/0.11.x");
}

#[test]
fn headers() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_header(UserAgent::new("hyper/0.11.x"));

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.headers.user_agent, "hyper/0.11.x");
}

#[test]
fn headers_clear() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_header(UserAgent::new("hyper/0.11.x"));

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.headers.user_agent, "hyper/0.11.x");

    client.clear_headers();

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.headers.user_agent, "");
}
