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
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     // Create new client with API base URL
//!     let mut client = RestClient::new("http://httpbin.org").unwrap();
//!
//!     // GET http://httpbin.org/anything and deserialize the result automatically
//!     let data = client.get::<_, HttpBinAnything>(()).await.unwrap().into_inner();
//!     println!("{:?}", data);
//! }
//! ```

use tokio::time::timeout;
use hyper::header::*;
use hyper::body::Buf;
use hyper::{Client, Method, Request};
use hyper::service::Service;
use hyper::client::connect::{dns, HttpConnector};
use log::{debug, trace, error};
use std::{error, fmt};
use std::ops::Deref;
use std::time::Duration;
use url::Url;

#[cfg(feature = "native-tls")]
use hyper_tls::HttpsConnector;
#[cfg(feature = "rustls")]
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

use resolvers::GaiResolver;

pub mod resolvers;
#[cfg(feature = "blocking")]
pub mod blocking;

static VERSION: &str = env!("CARGO_PKG_VERSION");

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

pub type HyperClient<R> = Client<HttpsConnector<HttpConnector<R>>>;

/// Type returned by client query functions
#[derive(Debug)]
pub struct Response<T> {
    body: T,
    headers: HeaderMap,
}

impl<T> Response<T> {
    /// Unwraps the response, getting the owned inner body
    pub fn into_inner(self) -> T {
        self.body
    }

    /// Response headers sent by the server
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl Response<String> {
    /// Parse a response body
    fn parse<T: serde::de::DeserializeOwned>(self) -> Result<Response<T>, Error> {
        #[cfg(feature = "lib-serde-json")]
        {
            let Self { body, headers } = self;
            serde_json::from_str::<T>(&body)
                .map(|body| Response { body, headers })
                .map_err(|err| Error::DeserializeParseError(err, body))
        }

        #[cfg(feature = "lib-simd-json")]
        {
            let Self { mut body, headers } = self;
            simd_json::serde::from_str(&mut body)
                .map(|body| Response { body, headers })
                .map_err(|err| Error::DeserializeParseSimdJsonError(err, body))
        }
    }
}

impl<T> Deref for Response<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

/// REST client to make HTTP GET and POST requests.
pub struct RestClient<R = GaiResolver> {
    client: HyperClient<R>,
    baseurl: url::Url,
    auth: Option<String>,
    headers: HeaderMap,
    timeout: Duration,
    send_null_body: bool,
    body_wash_fn: fn(String) -> String,
}

/// Restson error return type.
#[derive(Debug)]
pub enum Error {
    /// HTTP client creation failed
    HttpClientError,

    /// Failed to parse final URL.
    UrlError,

    /// Failed to serialize struct to JSON (in POST).
    SerializeParseError(serde_json::Error),

    /// Failed to deserialize data to struct (in GET or POST response).
    DeserializeParseError(serde_json::Error, String),

    /// Failed to deserialize data to struct from simd_json crate (in GET or POST response).
    #[cfg(feature = "lib-simd-json")]
    DeserializeParseSimdJsonError(simd_json::Error, String),

    /// Failed to make the outgoing request.
    RequestError,

    /// Failed to perform HTTP call using Hyper
    HyperError(hyper::Error),

    /// Failed to perform IO operation
    IoError(std::io::Error),

    /// Server returned non-success status.
    HttpError(u16, String),

    /// Request has timed out
    TimeoutError,

    /// Invalid parameter value
    InvalidValue,
}

/// Builder for `RestClient`
pub struct Builder<R = GaiResolver> {
    /// Request timeout
    timeout: Duration,

    /// Send null body
    send_null_body: bool,

    /// Hyper client to use for the connection
    client: Option<HyperClient<R>>,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            Error::HttpClientError => "HTTP Client creation failed",
            Error::UrlError => "Failed to parse final URL",
            Error::SerializeParseError(_) => "Failed to serialize struct to JSON (in POST)",
            Error::DeserializeParseError(_, _) => {
                "Failed to deserialize data to struct (in GET or POST)"
            }
            #[cfg(feature = "lib-simd-json")]
            Error::DeserializeParseSimdJsonError(_, _) => {
                "Failed to deserialize data to struct by simd_json crate (in GET or POST)"
            }
            Error::RequestError => "Failed to make the outgoing request",
            Error::HyperError(_) => "Failed to make the outgoing request due to Hyper error",
            Error::IoError(_) => "Failed to make the outgoing request due to IO error",
            Error::HttpError(_, _) => "Server returned non-success status",
            Error::TimeoutError => "Request has timed out",
            Error::InvalidValue => "Invalid parameter value",
        };
        fmt.write_str(desc)?;
        match *self {
            Error::SerializeParseError(ref err) => write!(fmt, ": {err}"),
            Error::DeserializeParseError(ref err, _) => write!(fmt, ": {err}"),
            #[cfg(feature = "lib-simd-json")]
            Error::DeserializeParseSimdJsonError(ref err, _) => write!(fmt, ": {err}"),
            Error::HyperError(ref err) => write!(fmt, ": {err}"),
            Error::IoError(ref err) => write!(fmt, ": {err}"),
            Error::HttpError(status, ref body) => write!(fmt, ": HTTP status {status}: {body}"),
            _ => Ok(()),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::SerializeParseError(ref err) => Some(err),
            Error::DeserializeParseError(ref err, _) => Some(err),
            Error::HyperError(ref err) => Some(err),
            #[cfg(feature = "lib-simd-json")]
            Error::DeserializeParseSimdJsonError(ref err, _) => Some(err),
            _ => None,
        }
    }
}

impl std::convert::From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::HyperError(e)
    }
}

impl std::convert::From<tokio::time::error::Elapsed> for Error {
    fn from(_e: tokio::time::error::Elapsed) -> Self {
        Error::TimeoutError
    }
}

impl<R: Service<dns::Name> + Clone> Default for Builder<R> {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(std::u64::MAX),
            send_null_body: true,
            client: None,
        }
    }
}

impl<R> Builder<R>
where
    R: Service<dns::Name> + Send + Sync + Default + Clone + 'static,
    HttpsConnector<HttpConnector<R>>: hyper::client::connect::Connect,
{
    /// Set request timeout
    ///
    /// Default is no timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send null body in POST/PUT
    ///
    /// Default is yes
    #[inline]
    pub fn send_null_body(mut self, value: bool) -> Self {
        self.send_null_body = value;
        self
    }

    pub fn with_client(mut self, client: HyperClient<R>) -> Self {
        self.client = Some(client);
        self
    }

    /// Create `RestClient` with the configuration in this builder
    pub fn build(self, url: &str) -> Result<RestClient<R>, Error> {
        RestClient::with_builder(url, self)
    }

    #[cfg(feature = "blocking")]
    /// Create [`blocking::RestClient`](blocking/struct.RestClient.html) with the configuration in
    /// this builder
    pub fn blocking(self, url: &str) -> Result<blocking::RestClient<R>, Error> {
        RestClient::with_builder(url, self).and_then(|client| client.try_into())
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

impl RestClient<GaiResolver> {
    /// Construct new client with default configuration and DNS resolver
    /// implementation to make HTTP requests.
    ///
    /// Use `Builder` to configure the client or to use a different
    /// resolver type.
    pub fn new(url: &str) -> Result<RestClient<GaiResolver>, Error> {
        RestClient::with_builder(url, Self::builder())
    }

    /// Construct new blocking client with default configuration and DNS resolver
    /// implementation to make HTTP requests.
    ///
    /// Use `Builder` to configure the client or to use a different
    /// resolver type.
    #[cfg(feature = "blocking")]
    pub fn new_blocking(url: &str) -> Result<blocking::RestClient<GaiResolver>, Error> {
        RestClient::new(url).and_then(|client| client.try_into())
    }
}

impl<R> RestClient<R>
where
    R: Service<dns::Name> + Send + Sync + Default + Clone + 'static,
    HttpsConnector<HttpConnector<R>>: hyper::client::connect::Connect,
{
    #[cfg(feature = "native-tls")]
    fn build_client() -> HyperClient<R>
    {
        let http_connector = HttpConnector::new_with_resolver(R::default());
        let https_connector = HttpsConnector::new_with_connector(http_connector);
        Client::builder().build(https_connector)
    }

    #[cfg(feature = "rustls")]
    fn build_client() -> HyperClient<R>
    {
        let http_connector = HttpConnector::new_with_resolver(R::default());
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_all_versions()
            .wrap_connector(http_connector);
        Client::builder().build(https_connector)
    }

    fn with_builder(url: &str, builder: Builder<R>) -> Result<RestClient<R>, Error> {
        let client = match builder.client {
            Some(client) => client,
            None => {
                Self::build_client()
            }
        };

        let baseurl = Url::parse(url).map_err(|_| Error::UrlError)?;

        debug!("new client for {}", baseurl);
        Ok(RestClient {
            client,
            baseurl,
            auth: None,
            headers: HeaderMap::new(),
            timeout: builder.timeout,
            send_null_body: builder.send_null_body,
            body_wash_fn: std::convert::identity,
        })
    }

    /// Configure a client
    pub fn builder() -> Builder<R> {
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
        s.push(':');
        s.push_str(pass);
        self.auth = Some("Basic ".to_owned() + &base64::encode(&s));
    }

    /// Set a function that cleans the response body up before deserializing it.
    pub fn set_body_wash_fn(&mut self, func: fn(String) -> String) {
        self.body_wash_fn = func;
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
        let value = HeaderValue::from_str(value).map_err(|_| Error::InvalidValue)?;
        self.headers.insert(name, value);
        Ok(())
    }

    /// Clear all previously set headers
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }

    /// Make a GET request.
    pub async fn get<U, T>(&self, params: U) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        let req = self.make_request::<U, T>(Method::GET, params, None, None)?;
        let res = self.run_request(req).await?;

        res.parse()
    }

    /// Make a GET request with query parameters.
    pub async fn get_with<U, T>(&self, params: U, query: &Query<'_>) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        let req = self.make_request::<U, T>(Method::GET, params, Some(query), None)?;
        let res = self.run_request(req).await?;

        res.parse()
    }

    /// Make a POST request.
    pub async fn post<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put(Method::POST, params, data).await
    }

    /// Make a PUT request.
    pub async fn put<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put(Method::PUT, params, data).await
    }

    /// Make a PATCH request.
    pub async fn patch<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put(Method::PATCH, params, data).await
    }

    async fn post_or_put<U, T>(&self, method: Method, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        let data = serde_json::to_string(data).map_err(Error::SerializeParseError)?;

        let req = self.make_request::<U, T>(method, params, None, Some(data))?;
        let res = self.run_request(req).await?;
        Ok(Response { body: (), headers: res.headers })
    }

    /// Make POST request with query parameters.
    pub async fn post_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put_with(Method::POST, params, data, query).await
    }

    /// Make PUT request with query parameters.
    pub async fn put_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put_with(Method::PUT, params, data, query).await
    }

    /// Make PATCH request with query parameters.
    pub async fn patch_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.post_or_put_with(Method::PATCH, params, data, query).await
    }

    async fn post_or_put_with<U, T>(
        &self,
        method: Method,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        let data = serde_json::to_string(data).map_err(Error::SerializeParseError)?;

        let req = self.make_request::<U, T>(method, params, Some(query), Some(data))?;
        let res = self.run_request(req).await?;
        Ok(Response { body: (), headers: res.headers })
    }

    /// Make a POST request and capture returned body.
    pub async fn post_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture(Method::POST, params, data).await
    }

    /// Make a PUT request and capture returned body.
    pub async fn put_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture(Method::PUT, params, data).await
    }

    /// Make a PATCH request and capture returned body.
    pub async fn patch_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture(Method::PATCH, params, data).await
    }

    /// Make a DELETE request and capture returned body.
    pub async fn delete_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture(Method::DELETE, params, data).await
    }

    async fn generic_capture<U, T, K>(
        &self,
        method: Method,
        params: U,
        data: &T,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        let data = serde_json::to_string(data).map_err(Error::SerializeParseError)?;

        let req = self.make_request::<U, T>(method, params, None, Some(data))?;
        let res = self.run_request(req).await?;
        res.parse()
    }

    /// Make a POST request with query parameters and capture returned body.
    pub async fn post_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture_with(Method::POST, params, data, query).await
    }

    /// Make a PUT request with query parameters and capture returned body.
    pub async fn put_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture_with(Method::PUT, params, data, query).await
    }

    /// Make a PATCH request with query parameters and capture returned body.
    pub async fn patch_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture_with(Method::PATCH, params, data, query).await
    }

    /// Make a DELETE request with query parameters and capture returned body.
    pub async fn delete_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.generic_capture_with(Method::DELETE, params, data, query).await
    }

    async fn generic_capture_with<U, T, K>(
        &self,
        method: Method,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        let data = serde_json::to_string(data).map_err(Error::SerializeParseError)?;

        let req = self.make_request::<U, T>(method, params, Some(query), Some(data))?;
        let res = self.run_request(req).await?;
        res.parse()
    }

    /// Make a DELETE request.
    pub async fn delete<U, T>(&self, params: U) -> Result<Response<()>, Error>
    where
        T: RestPath<U>,
    {
        let req = self.make_request::<U, T>(Method::DELETE, params, None, None)?;
        let res = self.run_request(req).await?;
        Ok(Response { body: (), headers: res.headers })
    }

    /// Make a DELETE request with query and body.
    pub async fn delete_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        let data = serde_json::to_string(data).map_err(Error::SerializeParseError)?;
        let req = self.make_request::<U, T>(Method::DELETE, params, Some(query), Some(data))?;
        let res = self.run_request(req).await?;
        Ok(Response { body: (), headers: res.headers })
    }

    async fn run_request(&self, req: hyper::Request<hyper::Body>) -> Result<Response<String>, Error> {
        debug!("{} {}", req.method(), req.uri());
        trace!("{:?}", req);

        let duration = self.timeout;
        let work = async {
            let res = self.client.request(req).await?;

            let response_headers = res.headers().clone();
            let status = res.status();
            let mut body = hyper::body::aggregate(res).await?;
            let body = body.copy_to_bytes(body.remaining());

            let body = String::from_utf8_lossy(&body);

            Ok::<_, hyper::Error>((response_headers, body.to_string(), status))
        };

        let res = if duration != Duration::from_secs(std::u64::MAX) {
            timeout(duration, work).await??
        } else {
            work.await?
        };

        let (response_headers, body, status) = res;

        if !status.is_success() {
            error!("server returned \"{}\" error", status);
            return Err(Error::HttpError(status.as_u16(), body));
        }

        trace!("response headers: {:?}", response_headers);
        trace!("response body: {}", body);
        Ok(Response { body: (self.body_wash_fn)(body), headers: response_headers })
    }

    fn make_request<U, T>(
        &self,
        method: Method,
        params: U,
        query: Option<&Query>,
        body: Option<String>,
    ) -> Result<Request<hyper::Body>, Error>
    where
        T: RestPath<U>,
    {
        let uri = self.make_uri(T::get_path(params)?.as_str(), query)?;
        let mut req = Request::new(hyper::Body::empty());

        *req.method_mut() = method;
        *req.uri_mut() = uri;

        if let Some(body) = body {
            if self.send_null_body || body != "null" {
                let len = HeaderValue::from_str(&body.len().to_string())
                    .map_err(|_| Error::RequestError)?;
                req.headers_mut().insert(CONTENT_LENGTH, len);
                req.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
                trace!("set request body: {}", body);
                *req.body_mut() = hyper::Body::from(body);
            }
        }

        if let Some(ref auth) = self.auth {
            req.headers_mut().insert(
                AUTHORIZATION,
                HeaderValue::from_str(auth).map_err(|_| Error::RequestError)?,
            );
        };

        for (key, value) in self.headers.iter() {
            req.headers_mut().insert(key, value.clone());
        }

        if !req.headers().contains_key(USER_AGENT) {
            req.headers_mut().insert(
                USER_AGENT,
                HeaderValue::from_str(&("restson/".to_owned() + VERSION))
                    .map_err(|_| Error::RequestError)?,
            );
        }

        Ok(req)
    }

    fn make_uri(&self, path: &str, params: Option<&Query>) -> Result<hyper::Uri, Error> {
        let mut url = self.baseurl.clone()
            .join(path)
            .map_err(|_| Error::UrlError)?;

        if let Some(params) = params {
            for &(key, item) in params.iter() {
                url.query_pairs_mut().append_pair(key, item);
            }
        }

        url.as_str()
            .parse::<hyper::Uri>()
            .map_err(|_| Error::UrlError)
    }
}
