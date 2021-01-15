use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HttpBinPost {
    data: String,
}

#[derive(Deserialize)]
struct HttpBinPostResp {
    json: HttpBinPost,
    url: String,
}

impl RestPath<()> for HttpBinPost {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("post"))
    }
}

#[tokio::test]
async fn basic_post() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPost {
        data: String::from("test data"),
    };
    client.post((), &data).await.unwrap();
}

#[tokio::test]
async fn post_query_params() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPost {
        data: String::from("test data"),
    };
    client.post_with((), &data, &params).await.unwrap();
}

#[tokio::test]
async fn post_capture() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPost {
        data: String::from("test data"),
    };
    let resp = client.post_capture::<_, _, HttpBinPostResp>((), &data).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/post");
}

#[tokio::test]
async fn post_capture_query_params() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPost {
        data: String::from("test data"),
    };
    let resp = client.post_capture_with::<_, _, HttpBinPostResp>((), &data, &params).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/post?a=2&b=abcd");
}
