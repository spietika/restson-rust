use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HttpBinPatch {
    data: String,
}

impl RestPath<()> for HttpBinPatch {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("patch"))
    }
}

#[tokio::test]
async fn basic_patch() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    client.patch((), &data).await.unwrap();
}

#[tokio::test]
async fn patch_query_params() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinPatch {
        data: String::from("test data"),
    };
    client.patch_with((), &data, &params).await.unwrap();
}
