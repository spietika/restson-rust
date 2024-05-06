use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HttpBinDelete {
    data: String,
}

#[derive(Deserialize)]
struct HttpBinDeleteResp {
    json: HttpBinDelete,
    url: String,
}

impl RestPath<()> for HttpBinDelete {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("delete"))
    }
}


#[test]
fn basic_delete() {
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    client.delete::<(), HttpBinDelete>(()).unwrap();
}

#[test]
fn delete_with() {
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinDelete {
        data: String::from("test data"),
    };
    client.delete_with((), &data, &params).unwrap();

    client.delete_with((), &data, &vec![]).unwrap();
}

#[test]
fn delete_capture() {
    let client = RestClient::new_blocking("https://httpbin.org").unwrap();

    let data = HttpBinDelete {
        data: String::from("test data"),
    };
    let resp = client.delete_capture::<_, _, HttpBinDeleteResp>((), &data).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/delete");
}

#[test]
fn delete_capture_query_params() {
    let client = RestClient::new_blocking("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinDelete {
        data: String::from("test data"),
    };
    let resp = client.delete_capture_with::<_, _, HttpBinDeleteResp>((), &data, &params).unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/delete?a=2&b=abcd");
}