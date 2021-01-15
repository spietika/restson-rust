//! Blocking variant of the `RestClient`

use crate::{Error, Query, Response, RestClient as AsyncRestClient, RestPath};
use hyper::header::HeaderValue;
use std::time::Duration;

/// REST client to make HTTP GET and POST requests. Blocking version.
pub struct RestClient {
    inner_client: AsyncRestClient,
}

impl From<AsyncRestClient> for RestClient {
    fn from(other: AsyncRestClient) -> Self {
        Self { inner_client: other }
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
    #[tokio::main(flavor = "current_thread")]
    pub async fn get<U, T>(&self, params: U) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        self.inner_client.get(params).await
    }

    /// Make a GET request with query parameters.
    #[tokio::main(flavor = "current_thread")]
    pub async fn get_with<U, T>(&self, params: U, query: &Query<'_>) -> Result<Response<T>, Error>
    where
        T: serde::de::DeserializeOwned + RestPath<U>,
    {
        self.inner_client.get_with(params, query).await
    }

    /// Make a POST request.
    #[tokio::main(flavor = "current_thread")]
    pub async fn post<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.post(params, data).await
    }

    /// Make a PUT request.
    #[tokio::main(flavor = "current_thread")]
    pub async fn put<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.put(params, data).await
    }

    /// Make a PATCH request.
    #[tokio::main(flavor = "current_thread")]
    pub async fn patch<U, T>(&self, params: U, data: &T) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.patch(params, data).await
    }

    /// Make POST request with query parameters.
    #[tokio::main(flavor = "current_thread")]
    pub async fn post_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.post_with(params, data, query).await
    }

    /// Make PUT request with query parameters.
    #[tokio::main(flavor = "current_thread")]
    pub async fn put_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.put_with(params, data, query).await
    }

    /// Make PATCH request with query parameters.
    #[tokio::main(flavor = "current_thread")]
    pub async fn patch_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.patch_with(params, data, query).await
    }

    /// Make a POST request and capture returned body.
    #[tokio::main(flavor = "current_thread")]
    pub async fn post_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.inner_client.post_capture(params, data).await
    }

    /// Make a PUT request and capture returned body.
    #[tokio::main(flavor = "current_thread")]
    pub async fn put_capture<U, T, K>(&self, params: U, data: &T) -> Result<Response<K>, Error>
    where
        T: serde::Serialize + RestPath<U>,
        K: serde::de::DeserializeOwned,
    {
        self.inner_client.put_capture(params, data).await
    }

    /// Make a POST request with query parameters and capture returned body.
    #[tokio::main(flavor = "current_thread")]
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
        self.inner_client.post_capture_with(params, data, query).await
    }

    /// Make a PUT request with query parameters and capture returned body.
    #[tokio::main(flavor = "current_thread")]
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
        self.inner_client.put_capture_with(params, data, query).await
    }

    /// Make a DELETE request.
    #[tokio::main(flavor = "current_thread")]
    pub async fn delete<U, T>(&self, params: U) -> Result<Response<()>, Error>
    where
        T: RestPath<U>,
    {
        self.inner_client.delete::<U, T>(params).await
    }

    /// Make a DELETE request with query and body.
    #[tokio::main(flavor = "current_thread")]
    pub async fn delete_with<U, T>(&self, params: U, data: &T, query: &Query<'_>) -> Result<Response<()>, Error>
    where
        T: serde::Serialize + RestPath<U>,
    {
        self.inner_client.delete_with(params, data, query).await
    }
}
