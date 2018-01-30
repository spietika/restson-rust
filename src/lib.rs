//! 
//! Easy-to-use REST client for Rust programming language that provides
//! automatic serialization and deserialization from Rust structs. The library
//! is implemented using [Hyper](https://github.com/hyperium/hyper) and 
//! [Serde JSON](https://github.com/serde-rs/json).
//! 
//! # Examples
//! ```
//! extern crate restson;
//! #[macro_use]
//! extern crate serde_derive;
//! 
//! use restson::{RestClient,RestPath,Error};
//! 
//! // Data structure that matches with REST API JSON
//! #[derive(Serialize,Deserialize,Debug)]
//! struct HttpBinAnything {
//!     method: String,
//!     url: String,
//! }
//! 
//! // Path of the REST endpoint: e.g. http://<baseurl>/anything
//! impl RestPath<()> for HttpBinAnything {
//!     fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("anything")) }
//! }
//!
//! fn main() {
//!     // Create new client with API base URL
//!     let mut client = RestClient::new("http://httpbin.org").unwrap();
//! 
//!     // GET http://httpbin.org/anything and deserialize the result automatically
//!     let data: HttpBinAnything = client.get(()).unwrap();
//!     println!("{:?}", data);
//! }
//! ```

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate url;
#[macro_use] 
extern crate log;

use futures::Future;
use futures::stream::Stream;
use hyper::{Client,Request,Method,StatusCode};
use hyper::header::*;
use hyper_tls::HttpsConnector;
use url::Url;

/// Type for URL query parameters. 
///
/// Slice of tuples in which the first field is parameter name and second is value.
/// These parameters are used with `get_with` and `post_with` functions.
///
/// # Examples
/// The vector
/// ```ignore
/// vec![("param1", "1234"), ("param2", "abcd")]
/// ```
/// would be parsed to **param1=1234&param2=abcd** in the request URL.
pub type Query<'a> = [(&'a str, &'a str)];

/// REST client to make HTTP GET and POST requests.
pub struct RestClient {
    core: tokio_core::reactor::Core,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    baseurl: url::Url,
    auth: Option<Authorization<Basic>>,
    headers: Headers,
}

/// Restson error return type.
#[derive(Debug)]
pub enum Error {
    /// HTTP client creation failed
    HttpClientError,

    /// Failed to parse final URL.
    UrlError,

    /// Failed to deserialize data to struct (in GET) or failed to 
    /// serialize struct to JSON (in POST).
    ParseError,

    /// Failed to make the outgoing request.
    RequestError,

    /// Server returned non-OK status.
    HttpError(u16),
}

/// Rest path builder trait for type.
///
/// Provides implementation for `rest_path` function that builds
/// type (and REST endpoint) specific API path from given parameter(s).
/// The built REST path is appended to the base URL given to `RestClient`.
/// If `Err` is returned, it is propagated directly to API caller.
pub trait RestPath<T> {
    /// Construct type specific REST API path from given parameters 
    /// (e.g. "api/devices/1234").
    fn get_path(par: T) -> Result<String, Error>;
}


impl RestClient {
    /// Construct new client to make HTTP requests.
    pub fn new(url: &str) -> Result<RestClient, Error> {
        let core = tokio_core::reactor::Core::new().map_err(|_| Error::HttpClientError)?;

        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).map_err(|_| Error::HttpClientError)?)
            .build(&handle);

        let baseurl = Url::parse(url).map_err(|_| Error::UrlError)?;

        debug!("new client for {}", baseurl);
        Ok(RestClient {
            core,
            client,
            baseurl,
            auth: None,
            headers: Headers::new(),
        })
    }

    /// Set credentials for HTTP Basic authentication.
    pub fn set_auth(&mut self, user: &str, pass: &str) { 
        self.auth = Some(Authorization(
            Basic {
                username: user.to_owned(),
                password: Some(pass.to_owned())
        }));
    }

    /// Set HTTP header from string name and value.
    ///
    /// The header is added to all subsequent GET and POST requests
    /// unless the headers are cleared with `clear_headers()` call.
    pub fn set_header_raw(&mut self, name: &str, value: &str) {
        self.headers.set_raw(name.to_owned(), value)
    }

    /// Set HTTP header from hyper Header.
    ///
    /// The header is added to all subsequent GET and POST requests
    /// unless the headers are cleared with `clear_headers()` call.
    pub fn set_header<H: Header>(&mut self, header: H) {
        self.headers.set(header)
    }

    /// Clear all previously set headers
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }

    /// Make a GET request.
    pub fn get<U, T>(&mut self, params: U) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {

        let uri = self.make_uri(T::get_path(params)?.as_str(), None)?;
        let body = self.run_request(Request::new(Method::Get, uri))?;

        serde_json::from_str(body.as_str()).map_err(|_| Error::ParseError)
    }

    /// Make a GET request with query parameters.
    pub fn get_with<U, T>(&mut self, params: U, query: &Query) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {
        let uri = self.make_uri(T::get_path(params)?.as_str(), Some(query))?;
        let body = self.run_request(Request::new(Method::Get, uri))?;

        serde_json::from_str(body.as_str()).map_err(|_| Error::ParseError)
    }

    /// Make a POST request.
    pub fn post<U, T>(&mut self, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let uri = self.make_uri(T::get_path(params)?.as_str(), None)?;

        let data = serde_json::to_string(data).map_err(|_| Error::ParseError)?;

        self.run_post_request(data, uri)?;
        Ok(())
    }

    /// Make POST request with query parameters.
    pub fn post_with<U, T>(&mut self, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let uri = self.make_uri(T::get_path(params)?.as_str(), Some(query))?;

        let data = serde_json::to_string(data).map_err(|_| Error::ParseError)?;
        
        self.run_post_request(data, uri)?;
        Ok(())
    }

    /// Make a POST request and capture returned body.
    pub fn post_capture<U, T, K>(&mut self, params: U, data: &T) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        let uri = self.make_uri(T::get_path(params)?.as_str(), None)?;

        let data = serde_json::to_string(data).map_err(|_| Error::ParseError)?;

        let body = self.run_post_request(data, uri)?;
        serde_json::from_str(body.as_str()).map_err(|_| Error::ParseError)
    }

    /// Make a POST request with query parameters and capture returned body.
    pub fn post_capture_with<U, T, K>(&mut self, params: U, data: &T, query: &Query) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        let uri = self.make_uri(T::get_path(params)?.as_str(), Some(query))?;

        let data = serde_json::to_string(data).map_err(|_| Error::ParseError)?;

        let body = self.run_post_request(data, uri)?;
        serde_json::from_str(body.as_str()).map_err(|_| Error::ParseError)
    }

    fn make_uri(&self, path: &str, params: Option<&Query>) -> Result<hyper::Uri, Error> {
        let mut url = self.baseurl.clone();
        url.set_path(path);

        if let Some(params) = params {
            for &(key, item) in params.iter() {
                url.query_pairs_mut().append_pair(key, item);
            }
        }

        url.as_str().parse::<hyper::Uri>().map_err(|_| Error::UrlError)
    }

    fn run_request(&mut self, mut req: hyper::Request) -> Result<String, Error> {
        if let Some(ref auth) = self.auth {
            req.headers_mut().set(auth.clone());
        };

        req.headers_mut().extend(self.headers.iter());

        debug!("{} {}", req.method(), req.uri());
        trace!("{:?}", req);

        let req = self.client.request(req).and_then(|res| {
            trace!("response headers: {:?}", res.headers());

            let status = Box::new(res.status());
            res.body().map(|chunk| {
                String::from_utf8_lossy(&chunk).to_string()
            }).collect().map(|vec| {
                (status, vec.into_iter().collect())
            })
        });

        match self.core.run(req) {
            Ok((status, body)) => {
                if *status != StatusCode::Ok {
                    error!("server returned \"{}\" error", *status);
                    return Err(Error::HttpError( (*status).as_u16() ));
                }
                Ok(body)
            },
            Err(_) => Err(Error::RequestError)
        }
    }

    fn run_post_request(&mut self, data: String, uri: hyper::Uri) -> Result<String, Error> {
        let mut req: Request = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentLength(data.len() as u64));
        req.headers_mut().set(ContentType(hyper::mime::APPLICATION_JSON));

        trace!("set request body: {}", data);
        req.set_body(data);

        Ok(self.run_request(req)?)
    }
} 