//! Blocking variant of the `RestClient`

use crate::{Error, Query, Response, RestClient as AsyncRestClient, RestPath};
use hyper::header::HeaderValue;
use std::{convert::TryFrom, time::Duration};
use tokio::runtime::{Builder, Runtime};

/// REST client to make HTTP GET and POST requests. Blocking version.
pub struct RestClient {
    inner_client: AsyncRestClient,
    runtime: Runtime,
}

impl TryFrom<AsyncRestClient> for RestClient {
    type Error = Error;

    fn try_from(other: AsyncRestClient) -> Result<Self, Self::Error> {
        match Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => Ok(Self { inner_client: other, runtime }),
            Err(e) => Err(Error::IoError(e)),
        }
    }
}

impl RestClient {
    /// Set whether a message body consisting only 'null' (from serde serialization)
    /// is sent in POST/PUT
    pub fn set_send_null_body(&mut self, send_null: bool) {
        self.inner_client.send_null_body = send_null;
    }

    /// Set credentials for HTTP Basic authentication.
    pub fn set_auth(&mut self, user: &str, pass: &str) {
        let mut s: String = user.to_owned();
        s.push(':');
        s.push_str(pass);
        self.inner_client.auth = Some("Basic ".to_owned() + &base64::encode(&s));
    }

    /// Set a function that cleans the response body up before deserializing it.
    pub fn set_body_wash_fn(&mut self, func: fn(String) -> String) {
        self.inner_client.body_wash_fn = func;
    }

    /// Set request timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.inner_client.timeout = timeout;
    }

    /// Set HTTP header from string name and value.
    ///
    /// The header is added to all subsequent GET and POST requests
    /// unless the headers are cleared with `clear_headers()` call.
    pub fn set_header(&mut self, name: &'static str, value: &str) -> Result<(), Error> {
        let value = HeaderValue::from_str(value).map_err(|_| Error::InvalidValue)?;
        self.inner_client.headers.insert(name, value);
        Ok(())
    }

    /// Clear all previously set headers
    pub fn clear_headers(&mut self) {
        self.inner_client.headers.clear();
    }

    /// Make a GET request.
    pub fn get<U, T>(&self, params: U) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.get(params))
    }

    /// Make a GET request with query parameters.
    pub fn get_with<U, T>(&self, params: U, query: &Query<'_>) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.get_with(params, query))
    }

    /// Make a POST request.
    pub fn post<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.post(params, data))
    }

    /// Make a PUT request.
    pub fn put<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.put(params, data))
    }

    /// Make a PATCH request.
    pub fn patch<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.patch(params, data))
    }

    /// Make POST request with query parameters.
    pub fn post_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.post_with(params, data, query))
    }

    /// Make PUT request with query parameters.
    pub fn put_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.put_with(params, data, query))
    }

    /// Make PATCH request with query parameters.
    pub fn patch_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.patch_with(params, data, query))
    }

    /// Make a POST request and capture returned body.
    pub fn post_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.post_capture(params, data))
    }

    /// Make a PUT request and capture returned body.
    pub fn put_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.put_capture(params, data))
    }

    /// Make a PATCH request and capture returned body.
    pub fn patch_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.patch_capture(params, data))
    }

    /// Make a POST request with query parameters and capture returned body.
    pub fn post_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.post_capture_with(params, data, query))
    }

    /// Make a PUT request with query parameters and capture returned body.
    pub fn put_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.put_capture_with(params, data, query))
    }

    /// Make a PATCH request with query parameters and capture returned body.
    pub fn patch_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.patch_capture_with(params, data, query))
    }

    /// Make a DELETE request.
    pub fn delete<U, T>(&self, params: U) -> Result<Response<()>, Error>
    where
        T: RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.delete::<U, T>(params))
    }

    /// Make a DELETE request with query and body.
    pub fn delete_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.runtime.block_on(self.inner_client.delete_with(params, data, query))
    }

    /// Make a DELETE request and capture returned body.
    pub fn delete_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.delete_capture(params, data))
    }

    /// Make a DELETE request with query parameters and capture returned body.
    pub fn delete_capture_with<U, T, K>(
        &self,
        params: U,
        data: &T,
        query: &Query<'_>,
    ) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.runtime.block_on(self.inner_client.delete_capture_with(params, data, query))
    }
}
