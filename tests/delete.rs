extern crate restson;

#[macro_use]
extern crate serde_derive;

use restson::{Error, RestClient, RestPath};

#[derive(Serialize, Deserialize)]
struct HttpBinDelete {
    data: String,
}

impl RestPath<()> for HttpBinDelete {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("delete"))
    }
}


#[test]
fn basic_delete() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    client.delete::<(), HttpBinDelete>(()).unwrap();
}

#[test]
fn delete_with() {
    let mut client = RestClient::new("http://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinDelete {
        data: String::from("test data"),
    };
    client.delete_with((), &data, &params).unwrap();

    client.delete_with((), &data, &vec![]).unwrap();
}
