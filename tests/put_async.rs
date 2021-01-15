use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

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

#[tokio::test]
async fn basic_put() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPut {
        data: String::from("test data"),
    };
    client.put((), &data).await.unwrap();
}

#[tokio::test]
async fn put_query_params() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPut {
        data: String::from("test data"),
    };
    client.put_with((), &data, &params).await.unwrap();
}

#[tokio::test]
async fn put_capture() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPut {
        data: String::from("test data"),
    };
    let resp = client.put_capture::<_, _, HttpBinPutResp>((), &data).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/put");
}

#[tokio::test]
async fn put_capture_query_params() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPut {
        data: String::from("test data"),
    };
    let resp = client.put_capture_with::<_, _, HttpBinPutResp>((), &data, &params).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/put?a=2&b=abcd");
}
