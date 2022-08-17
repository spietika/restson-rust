use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HttpBinPatch {
    data: String,
}

#[derive(Deserialize)]
struct HttpBinPatchResp {
    json: HttpBinPatch,
    url: String,
}

impl RestPath<()> for HttpBinPatch {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("patch"))
    }
}

#[tokio::test]
async fn basic_patch() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    client.patch((), &data).await.unwrap();
}

#[tokio::test]
async fn patch_query_params() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    client.patch_with((), &data, &params).await.unwrap();
}

#[tokio::test]
async fn patch_capture() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    let resp = client.patch_capture::<_, _, HttpBinPatchResp>((), &data).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/patch");
}

#[tokio::test]
async fn patch_capture_query_params() {
    let client = RestClient::new("https://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    let resp =
        client.patch_capture_with::<_, _, HttpBinPatchResp>((), &data, &params).await.unwrap();

    assert_eq!(resp.json.data, "test data");
    assert_eq!(resp.url, "https://httpbin.org/patch?a=2&b=abcd");
}
