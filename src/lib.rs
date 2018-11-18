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
extern crate base64;

use hyper::rt::Future;
use hyper::rt::Stream;
use futures::future::Either;
use hyper::{Client,Request,Method};
use hyper::header::*;
use hyper_tls::HttpsConnector;
use url::Url;
use tokio_core::reactor::Timeout;
use std::time::Duration;
use std::error;
use std::fmt;

static VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
    auth: Option<String>,
    headers: HeaderMap,
    timeout: Duration,
    send_null_body: bool
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
    ParseError(serde_json::Error),

    /// Failed to make the outgoing request.
    RequestError,

    /// Server returned non-success status.
    HttpError(u16, String),

    /// Request has timed out
    TimeoutError,

    /// Invalid parameter value
    InvalidValue,
}

/// Builder for `RestClient`
pub struct Builder {
    /// Number of DNS worker threads
    dns_workers: usize
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(error::Error::description(self))?;
        match *self {
            Error::ParseError(ref err) => write!(fmt, ": {}", err),
            _ => Ok(())
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::HttpClientError => "HTTP Client creation failed",
            Error::UrlError => "Failed to parse final URL",
            Error::ParseError(_) => "Failed to deserialze data to struct (in GET) or failed to serialize struct to JSON (in POST)",
            Error::RequestError => "Failed to make the outgoing request",
            Error::HttpError(_, _) => "Server returned non-success status",
            Error::TimeoutError => "Request has timed out",
            Error::InvalidValue => "Invalid parameter value",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ParseError(ref err) => Some(err),
            _ => None
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            dns_workers: 4
        }
    }
}

impl Builder {
    /// Set number of DNS worker threads
    ///
    /// Default is 4
    #[inline]
    pub fn dns_workers(&mut self, workers: usize) -> &mut Self {
        self.dns_workers = workers;
        self
    }

    /// Create `RestClient` with the configuration in this builder
    pub fn build(&self, url: &str) -> Result<RestClient, Error> {
        RestClient::with_builder(url, self)
    }
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
    /// Construct new client with default configuration to make HTTP requests.
    ///
    /// Use `Builder` to configure the client.
    pub fn new(url: &str) -> Result<RestClient, Error> {
        RestClient::with_builder(url, &RestClient::builder())
    }

    fn with_builder(url: &str, builder: &Builder) -> Result<RestClient, Error> {
        let core = tokio_core::reactor::Core::new().map_err(|_| Error::HttpClientError)?;

        let https = HttpsConnector::new(builder.dns_workers).map_err(|_| Error::HttpClientError)?;
        let client = Client::builder().build(https);

        let baseurl = Url::parse(url).map_err(|_| Error::UrlError)?;

        debug!("new client for {}", baseurl);
        Ok(RestClient {
            core,
            client,
            baseurl,
            auth: None,
            headers: HeaderMap::new(),
            // For some reason using u32::MAX or u64::MAX causes request error inside
            // tokio/hyper. Use 1 year as default which is sufficiently large for "no timeout"
            timeout: Duration::from_secs(31_556_926),
            send_null_body: true,
        })
    }

    /// Configure a client
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Set whether a message body consisting only 'null' (from serde serialization)
    /// is sent in POST/PUT
    pub fn set_send_null_body(&mut self, send_null: bool) {
        self.send_null_body = send_null;
    }

    /// Set credentials for HTTP Basic authentication.
    pub fn set_auth(&mut self, user: &str, pass: &str) {
        let mut s: String = user.to_owned();
        s.push_str(":");
        s.push_str(pass);
        self.auth = Some("Basic ".to_owned() + &base64::encode(&s));
    }

    /// Set request timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Set HTTP header from string name and value.
    ///
    /// The header is added to all subsequent GET and POST requests
    /// unless the headers are cleared with `clear_headers()` call.
    pub fn set_header(&mut self, name: &'static str, value: &str) -> Result<(), Error> {
        let value =  HeaderValue::from_str(value).map_err(|_| Error::InvalidValue)?;
        self.headers.insert(name, value);
        Ok(())
    }

    /// Clear all previously set headers
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }

    /// Make a GET request.
    pub fn get<U, T>(&mut self, params: U) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {

        let req = self.make_request::<U,T>(Method::GET, params, None, None)?;
        let body = self.run_request(req)?;

        serde_json::from_str(body.as_str()).map_err(Error::ParseError)
    }

    /// Make a GET request with query parameters.
    pub fn get_with<U, T>(&mut self, params: U, query: &Query) -> Result<T, Error> where
        T: serde::de::DeserializeOwned + RestPath<U> {
        let req = self.make_request::<U,T>(Method::GET, params, Some(query), None)?;
        let body = self.run_request(req)?;

        serde_json::from_str(body.as_str()).map_err(Error::ParseError)
    }

    /// Make a POST request.
    pub fn post<U, T>(&mut self, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put(Method::POST, params, data)
    }

    /// Make a PUT request.
    pub fn put<U, T>(&mut self, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put(Method::PUT, params, data)
    }

    /// Make a PATCH request.
    pub fn patch<U, T>(&mut self, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put(Method::PATCH, params, data)
    }

    fn post_or_put<U, T>(&mut self, method: Method, params: U, data: &T) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let data = serde_json::to_string(data).map_err(Error::ParseError)?;

        let req = self.make_request::<U,T>(method, params, None, Some(data))?;
        self.run_request(req)?;
        Ok(())
    }

    /// Make POST request with query parameters.
    pub fn post_with<U, T>(&mut self, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put_with(Method::POST, params, data, query)
    }

    /// Make PUT request with query parameters.
    pub fn put_with<U, T>(&mut self, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put_with(Method::PUT, params, data, query)
    }

    /// Make PATCH request with query parameters.
    pub fn patch_with<U, T>(&mut self, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        self.post_or_put_with(Method::PATCH, params, data, query)
    }

    fn post_or_put_with<U, T>(&mut self, method: Method, params: U, data: &T, query: &Query) -> Result<(), Error> where 
        T: serde::Serialize + RestPath<U> {
        let data = serde_json::to_string(data).map_err(Error::ParseError)?;
        
        let req = self.make_request::<U,T>(method, params, Some(query), Some(data))?;
        self.run_request(req)?;
        Ok(())
    }

    /// Make a POST request and capture returned body.
    pub fn post_capture<U, T, K>(&mut self, params: U, data: &T) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        self.post_or_put_capture(Method::POST, params, data)
    }

    /// Make a PUT request and capture returned body.
    pub fn put_capture<U, T, K>(&mut self, params: U, data: &T) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        self.post_or_put_capture(Method::PUT, params, data)
    }

    fn post_or_put_capture<U, T, K>(&mut self, method: Method, params: U, data: &T) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        let data = serde_json::to_string(data).map_err(Error::ParseError)?;

        let req = self.make_request::<U,T>(method, params, None, Some(data))?;
        let body = self.run_request(req)?;
        serde_json::from_str(body.as_str()).map_err(Error::ParseError)
    }

    /// Make a POST request with query parameters and capture returned body.
    pub fn post_capture_with<U, T, K>(&mut self, params: U, data: &T, query: &Query) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        self.post_or_put_capture_with(Method::POST, params, data, query)
    }

    /// Make a PUT request with query parameters and capture returned body.
    pub fn put_capture_with<U, T, K>(&mut self, params: U, data: &T, query: &Query) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        self.post_or_put_capture_with(Method::PUT, params, data, query)
    }

    fn post_or_put_capture_with<U, T, K>(&mut self, method: Method, params: U, data: &T, query: &Query) -> Result<K, Error> where 
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned {
        let data = serde_json::to_string(data).map_err(Error::ParseError)?;

        let req = self.make_request::<U,T>(method, params, Some(query), Some(data))?;
        let body = self.run_request(req)?;
        serde_json::from_str(body.as_str()).map_err(Error::ParseError)
    }

    /// Make a DELETE request.
    pub fn delete<U, T>(&mut self, params: U) -> Result<(), Error> where
        T: RestPath<U> {

        let req = self.make_request::<U,T>(Method::DELETE, params, None, None)?;
        self.run_request(req)?;
        Ok(())
    }

    fn run_request(&mut self, req: hyper::Request<hyper::Body>) -> Result<String, Error> {
        debug!("{} {}", req.method(), req.uri());
        trace!("{:?}", req);

        let req = self.client.request(req).and_then(|res| {
            trace!("response headers: {:?}", res.headers());

            let status = Box::new(res.status());
            res.into_body().map(|chunk| {
                String::from_utf8_lossy(&chunk).to_string()
            }).collect().map(|vec| {
                (status, vec.into_iter().collect())
            })
        });

        let timeout = Timeout::new(self.timeout, &self.core.handle()).map_err(|_| Error::RequestError)?;
        let work = req.select2(timeout).then(|res| match res {
            Ok(Either::A((got, _))) => Ok(got),
            Ok(Either::B((_, _))) => Err(Error::TimeoutError),
            Err(_) => Err(Error::RequestError)
        });

        match self.core.run(work) {
            Ok((status, body)) => {
                let status = *status;
                if !status.is_success() {
                    error!("server returned \"{}\" error", status);
                    return Err(Error::HttpError( status.as_u16(), body ));
                }
                trace!("response body: {}", body);
                Ok(body)
            },
            Err(e) => Err(e)
        }
    }

    pub fn make_request<U, T>(&mut self, method: Method, params: U,
                               query: Option<&Query>, body: Option<String>) -> Result<Request<hyper::Body>,Error> where
        T: RestPath<U> {
        let uri = self.make_uri(T::get_path(params)?.as_str(), query)?;
        let mut req = Request::new(hyper::Body::empty());

        *req.method_mut() = method;
        *req.uri_mut() = uri.clone();

        if let Some(body) = body {
            if self.send_null_body || body != "null" {
                let len =  HeaderValue::from_str(&body.len().to_string()).map_err(|_| Error::RequestError)?;
                req.headers_mut().insert(CONTENT_LENGTH, len);
                req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
                trace!("set request body: {}", body);
                *req.body_mut() = hyper::Body::from(body);
            }
        }

        if let Some(ref auth) = self.auth {
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(auth).map_err(|_| Error::RequestError)?);
        };

        for (key, value) in self.headers.iter() {
            req.headers_mut().insert(key, value.clone());
        }

        if !req.headers().contains_key(USER_AGENT) {
            req.headers_mut().insert(USER_AGENT,
                HeaderValue::from_str(&("restson/".to_owned() + VERSION)).map_err(|_| Error::RequestError)?);
        }

        Ok(req)
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
}
