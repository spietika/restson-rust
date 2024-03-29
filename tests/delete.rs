use restson::{Error, RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

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
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    client.delete::<(), HttpBinDelete>(()).unwrap();
}

#[test]
fn delete_with() {
    let client = RestClient::new_blocking("http://httpbin.org").unwrap();

    let params = vec![("a", "2"), ("b", "abcd")];
    let data = HttpBinDelete {
        data: String::from("test data"),
    };
    client.delete_with((), &data, &params).unwrap();

    client.delete_with((), &data, &vec![]).unwrap();
}
