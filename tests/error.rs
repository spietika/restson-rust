extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,Error,RestPath};

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