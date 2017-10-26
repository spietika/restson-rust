extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate url;

use futures::Future;
use futures::stream::Stream;
use hyper::{Client,Request,Method};
use hyper::header::{Authorization,Basic};
use hyper_tls::HttpsConnector;
use url::Url;

pub type Query<'a> = Vec<(&'a str, &'a str)>;

pub struct RestClient {
    core: tokio_core::reactor::Core,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    baseurl: url::Url,
    auth: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    Internal,
}

pub trait RestPath<T> {
    fn get_path(par: T) -> String;
}


impl RestClient {
    pub fn new(url: &str) -> Result<RestClient, Error> {
        let core = tokio_core::reactor::Core::new().unwrap();

        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        Ok(RestClient {
            core,
            client,
            baseurl: Url::parse(url).unwrap(),
            auth: None
        })
    }

    pub fn set_auth(&mut self, user: &str, pass: &str) {
        let auth = Authorization(
            Basic {
                username: user.to_owned(),
                password: Some(pass.to_owned())
        });

        self.auth = Some(format!("{}", auth));   
    }

    pub fn get<U, T>(&mut self, params: U) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {

        let uri = self.make_uri(T::get_path(params).as_str(), None).unwrap();
        let body = self.run_request(Request::new(Method::Get, uri)).unwrap();

        Ok(serde_json::from_str(body.as_str()).unwrap())
    }

    pub fn get_with<U, T>(&mut self, params: U, query: &Query) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {
        let uri = self.make_uri(T::get_path(params).as_str(), Some(query)).unwrap();
        let body = self.run_request(Request::new(Method::Get, uri)).unwrap();

        Ok(serde_json::from_str(body.as_str()).unwrap())
    }

    pub fn post<U, T>(&mut self, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let uri = self.make_uri(T::get_path(params).as_str(), None).unwrap();

        let data = serde_json::to_string(data).unwrap();
        self.run_post_request(data, uri)
    }

    pub fn post_with<U, T>(&mut self, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let uri = self.make_uri(T::get_path(params).as_str(), Some(query)).unwrap();

        let data = serde_json::to_string(data).unwrap();
        self.run_post_request(data, uri)
    }

    fn make_uri(&self, path: &str, params: Option<&Query>) -> Result<hyper::Uri, hyper::error::UriError> {
        let mut url = self.baseurl.clone();
        url.set_path(path);

        if let Some(params) = params {
            for &(ref key, ref item) in params.iter() {
                url.query_pairs_mut().append_pair(key, item);
            }
        }

        Ok(url.as_str().parse::<hyper::Uri>()?)
    }

    fn run_request(&mut self, mut req: hyper::Request) -> Option<String> {
        if let &Some(ref auth) = &self.auth {
            req.headers_mut().set_raw("Authorization", format!("{}", auth));
        };

        let req = self.client.request(req).and_then(|res| {
            res.body().map(|chunk| {
                String::from_utf8_lossy(&chunk).to_string()
            }).collect()
        });

        match self.core.run(req) {
            Ok(data) => {
                let mut out = String::new();
                out.extend(data);
                Some(out)
            },
            Err(_) => None
        }
    }

    fn run_post_request(&mut self, data: String, uri: hyper::Uri) -> Result<(), Error> {
        let mut req: Request = Request::new(Method::Post, uri);
        req.headers_mut().set_raw("Content-Length", format!("{}", data.len()));
        req.headers_mut().set_raw("Content-Type", "application/json");
        req.set_body(data);

        self.run_request(req);

        Ok(())
    }
} 