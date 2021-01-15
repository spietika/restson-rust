use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
struct InvalidResource {}

impl RestPath<()> for InvalidResource {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("not_found"))
    }
}

impl RestPath<bool> for InvalidResource {
    fn get_path(param: bool) -> Result<String, Error> {
        if param {
            return Ok(String::from("path"));
        }
        Err(Error::UrlError)
    }
}

#[derive(Serialize, Deserialize)]
struct HttpBinStatus {}

impl RestPath<u16> for HttpBinStatus {
    fn get_path(code: u16) -> Result<String, Error> {
        Ok(format!("status/{}", code))
    }
}

#[derive(Serialize, Deserialize)]
struct HttpBinDelay {}

impl RestPath<u16> for HttpBinDelay {
    fn get_path(delay: u16) -> Result<String, Error> {
        Ok(format!("delay/{}", delay))
    }
}

#[derive(Serialize, Deserialize)]
struct HttpBinBase64 {}

impl RestPath<String> for HttpBinBase64 {
    fn get_path(data: String) -> Result<String, Error> {
        Ok(format!("base64/{}", data))
    }
}

#[tokio::test]
async fn invalid_get() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    if client.get::<(), InvalidResource>(()).await.is_ok() {
        panic!("expected error");
    }
}

#[tokio::test]
async fn invalid_post() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    let data = InvalidResource {};

    if client.post((), &data).await.is_ok() {
        panic!("expected error");
    }
}

#[tokio::test]
async fn path_error() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    if let Err(Error::UrlError) = client.get::<bool, InvalidResource>(false).await {
    } else {
        panic!("expected url error");
    }
}

#[tokio::test]
async fn http_error() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    match client.get::<_, HttpBinStatus>(418).await {
        Err(Error::HttpError(s, body)) => {
            assert_eq!(s, 418);
            assert!(!body.is_empty());
        }
        _ => panic!("Expected 418 error status with response body"),
    };
}

#[tokio::test]
async fn request_timeout() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_timeout(Duration::from_secs(1));

    let start = Instant::now();
    if let Err(Error::TimeoutError) = client.get::<u16, HttpBinDelay>(3).await {
        assert!(start.elapsed().as_secs() == 1);
    } else {
        panic!("expected timeout error");
    }
}

#[tokio::test]
#[cfg(feature = "lib-serde-json")]
async fn deserialize_error() {
    let client = RestClient::new("http://httpbin.org").unwrap();

    // Service returns decoded base64 in body which should be string 'test'.
    // This fails JSON deserialization and is returned in the Error
    if let Err(Error::DeserializeParseError(_, data)) =
        client.get::<String, HttpBinBase64>("dGVzdA==".to_string()).await
    {
        assert!(data == "test");
    } else {
        panic!("expected serialized error");
    }
}

#[tokio::test]
#[cfg(feature = "lib-simd-json")]
async fn deserialize_error() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    if let Err(Error::DeserializeParseSimdJsonError(_, data)) =
        client.get::<String, HttpBinBase64>("dGVzdA==".to_string()).await
    {
        assert!(data == "test");
    } else {
        panic!("expected serialized error");
    }
}