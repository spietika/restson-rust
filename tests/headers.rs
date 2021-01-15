use hyper::header::*;
use restson::{Error, RestClient, RestPath};
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct HttpBinAnything {
    headers: TestHeaders,
}

#[derive(Deserialize)]
struct TestHeaders {
    #[serde(default)]
    #[serde(rename = "User-Agent")]
    user_agent: String,

    #[serde(default)]
    #[serde(rename = "X-Test")]
    test: String,
}

impl RestPath<()> for HttpBinAnything {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("anything"))
    }
}

#[test]
fn headers() {
    let mut client = RestClient::new_blocking("http://httpbin.org").unwrap();

    client
        .set_header(USER_AGENT.as_str(), "restson-test")
        .unwrap();

    let data = client.get::<_, HttpBinAnything>(()).unwrap().into_inner();
    assert_eq!(data.headers.user_agent, "restson-test");
}

#[test]
fn headers_clear() {
    let mut client = RestClient::new_blocking("http://httpbin.org").unwrap();

    client.set_header("X-Test", "12345").unwrap();

    let data = client.get::<_, HttpBinAnything>(()).unwrap().into_inner();
    assert_eq!(data.headers.test, "12345");

    client.clear_headers();

    let data = client.get::<_, HttpBinAnything>(()).unwrap().into_inner();
    assert_eq!(data.headers.test, "");
}

#[test]
fn default_user_agent() {
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    let data = client.get::<_, HttpBinAnything>(()).unwrap().into_inner();
    assert_eq!(
        data.headers.user_agent,
        "restson/".to_owned() + env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn response_headers() {
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    let data = client.get::<_, HttpBinAnything>(()).unwrap();
    assert_eq!(data.headers()["content-type"], "application/json");
}
