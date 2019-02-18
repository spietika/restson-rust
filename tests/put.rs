extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{Error, RestClient, RestPath};

#[derive(Serialize, Deserialize)]
struct HttpBinPut {
    data: String,
}

#[derive(Deserialize)]
struct HttpBinPutResp {
    json: HttpBinPut,
    url: String,
}

impl RestPath<()> for HttpBinPut {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("put"))
    }
}

#[test]
fn basic_put() {
    let mut client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPut {
        data: String::from("test data"),
    };
    client.put((), &data).unwrap();
}

#[test]
fn put_query_params() {
    let mut client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPut {
        data: String::from("test data"),
    };
    client.put_with((), &data, &params).unwrap();
}

#[test]
fn put_capture() {
    let mut client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPut {
        data: String::from("test data"),
    };
    let resp: HttpBinPutResp = client.put_capture((), &data).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/put");
}

#[test]
fn put_capture_query_params() {
    let mut client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPut {
        data: String::from("test data"),
    };
    let resp: HttpBinPutResp = client.put_capture_with((), &data, &params).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/put?a=2&b=abcd");
}
