extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,Error,RestPath};

#[derive(Serialize,Deserialize)]
struct InvalidResource {
}

impl RestPath<()> for InvalidResource {
    fn get_path(_: ()) -> String { String::from("not_found") }
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