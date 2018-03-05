extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,Error,RestPath};
use std::time::{Duration, Instant};

#[derive(Serialize,Deserialize)]
struct InvalidResource {
}

impl RestPath<()> for InvalidResource {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("not_found")) }
}

impl RestPath<bool> for InvalidResource {
    fn get_path(param: bool) -> Result<String,Error> { 
        if param {
            return Ok(String::from("path"));
        }
        Err(Error::UrlError)
    }
}

#[derive(Serialize,Deserialize)]
struct HttpBinStatus {
}

impl RestPath<u16> for HttpBinStatus {
    fn get_path(code: u16) -> Result<String,Error> { Ok(format!("status/{}", code)) }
}

#[derive(Serialize,Deserialize)]
struct HttpBinDelay {
}

impl RestPath<u16> for HttpBinDelay {
    fn get_path(delay: u16) -> Result<String,Error> { Ok(format!("delay/{}", delay)) }
}


#[test]
fn invalid_baseurl() {
    match RestClient::new("1234") {
        Err(Error::UrlError) => (),
        _ => panic!("Expected url error")
    };
}

#[test]
fn invalid_get() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    if client.get::<(), InvalidResource>(()).is_ok() {
        panic!("expected error");
    }
}

#[test]
fn invalid_post() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let data = InvalidResource {};

    if client.post((), &data).is_ok() {
        panic!("expected error");
    }
}

#[test]
fn path_error() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    if let Err(Error::UrlError) = client.get::<bool, InvalidResource>(false) {
    }
    else {
        panic!("expected url error");
    }
}

#[test]
fn http_error() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    match client.get::<_, HttpBinStatus>(418) {
        Err(Error::HttpError(s, body)) => {
            assert_eq!(s, 418);
            assert!(!body.is_empty());
        },
        _ => panic!("Expected 418 error status with response body"), 
    };
}

#[test]
fn request_timeout() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_timeout(Duration::from_secs(1));

    let start = Instant::now();
    if let Err(Error::TimeoutError) = client.get::<u16, HttpBinDelay>(3) {
        assert!(start.elapsed().as_secs() == 1);
    }
    else {
        panic!("expected timeout error");
    }
}
