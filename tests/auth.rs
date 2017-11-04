extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{RestClient,RestPath};

#[derive(Deserialize)]
struct HttpBinBasicAuth {
}

impl<'a> RestPath<(&'a str,&'a str)> for HttpBinBasicAuth {
    fn get_path(auth: (&str,&str)) -> String { 
        let (user,pass) = auth;
        format!("basic-auth/{}/{}", user, pass) 
    }
}

#[test]
fn basic_auth() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.set_auth("username", "passwd");
    client.get::<_, HttpBinBasicAuth>(("username", "passwd")).unwrap();
}
