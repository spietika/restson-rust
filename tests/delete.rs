extern crate restson;

use restson::{RestClient,RestPath,Error};

struct HttpBinDelete {
}

impl RestPath<()> for HttpBinDelete {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("delete")) }
}

#[test]
fn basic_delete() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.delete::<(), HttpBinDelete>(()).unwrap();
}
