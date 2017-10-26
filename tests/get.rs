extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,RestPath};

#[derive(Deserialize)]
struct HttpBinAnything {
    url: String,
    args: HttpBinAnythingArgs,
}

#[derive(Deserialize)]
struct HttpBinAnythingArgs {
    #[serde(default)]
    a: String,
    #[serde(default)]
    b: String,
}

impl RestPath<()> for HttpBinAnything {
    fn get_path(_: ()) -> String { String::from("anything") }
}

impl RestPath<u32> for HttpBinAnything {
    fn get_path(param: u32) -> String { format!("anything/{}", param) }
}

impl<'a> RestPath<(u32, &'a str)> for HttpBinAnything {
    fn get_path(param: (u32, &str)) -> String { 
        let (a,b) = param;
        format!("anything/{}/{}", a, b)
    }
}

#[test]
fn basic_get_http() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.url, "http://httpbin.org/anything");
}

#[test]
fn basic_get_https() {
    let mut client = RestClient::new("https://httpbin.org").unwrap();

    let data: HttpBinAnything = client.get(()).unwrap();
    assert_eq!(data.url, "https://httpbin.org/anything");
}

#[test]
fn get_path_param() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data: HttpBinAnything = client.get(1234).unwrap();
    assert_eq!(data.url, "http://httpbin.org/anything/1234");
}

#[test]
fn get_multi_path_param() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data: HttpBinAnything = client.get((1234, "abcd")).unwrap();
    assert_eq!(data.url, "http://httpbin.org/anything/1234/abcd");
}

#[test]
fn get_query_params() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a","2"), ("b","abcd")];
    let data: HttpBinAnything = client.get_with((), &params).unwrap();

    assert_eq!(data.url, "http://httpbin.org/anything?a=2&b=abcd");
    assert_eq!(data.args.a, "2");
    assert_eq!(data.args.b, "abcd");
}
