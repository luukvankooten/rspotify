//! The client implementation for the ureq HTTP client, which is blocking.

use super::{BaseHttpClient, Form, Headers, Query};
use crate::client::{ClientError, ClientResult};

use maybe_async::sync_impl;
use serde_json::Value;
use ureq::{Request, Response};

impl ClientError {
    pub fn from_response(r: ureq::Response) -> Self {
        ClientError::StatusCode(r.status(), r.status_text().to_string())
    }
}

#[derive(Default, Debug, Clone)]
pub struct UreqClient {}

impl UreqClient {
    /// The request handling in ureq is split in three parts:
    ///
    /// * The initial request (POST, GET, ...) is given as the `request`
    ///   parameter.
    /// * This method will add whichever headers and additional data is needed
    ///   for all requests.
    /// * The request is finished and performed with the `send_request` function
    ///   (JSON, a form...).
    fn request<D>(
        &self,
        mut request: Request,
        headers: Option<&Headers>,
        send_request: D,
    ) -> ClientResult<String>
    where
        D: Fn(Request) -> Result<Response, ureq::Error>,
    {
        // Setting the headers, which will be the token auth if unspecified.
        if let Some(headers) = headers {
            for (key, val) in headers.iter() {
                request = request.set(&key, &val);
            }
        }

        log::info!("Making request {:?}", request);
        match send_request(request) {
            // Successful request
            Ok(response) => response.into_string().map_err(Into::into),
            // HTTP status error
            Err(ureq::Error::Status(_, response)) => Err(ClientError::from_response(response)),
            // Some kind of IO/transport error
            Err(err) => Err(ClientError::Request(err.to_string())),
        }
    }
}

#[sync_impl]
impl BaseHttpClient for UreqClient {
    #[inline]
    fn get(&self, url: &str, headers: Option<&Headers>, payload: &Query) -> ClientResult<String> {
        let request = ureq::get(url);
        let sender = |mut req: Request| {
            for (key, val) in payload.iter() {
                req = req.query(&key, &val)
            }
            req.call()
        };
        self.request(request, headers, sender)
    }

    #[inline]
    fn post(&self, url: &str, headers: Option<&Headers>, payload: &Value) -> ClientResult<String> {
        let request = ureq::post(url);
        let sender = |req: Request| req.send_json(payload.clone());
        self.request(request, headers, sender)
    }

    #[inline]
    fn post_form<'a>(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &Form<'a>,
    ) -> ClientResult<String> {
        let request = ureq::post(url);
        let sender = |req: Request| {
            let payload = payload
                .iter()
                .map(|(key, val)| (*key, *val))
                .collect::<Vec<_>>();

            req.send_form(&payload)
        };

        self.request(request, headers, sender)
    }

    #[inline]
    fn put(&self, url: &str, headers: Option<&Headers>, payload: &Value) -> ClientResult<String> {
        let request = ureq::put(url);
        let sender = |req: Request| req.send_json(payload.clone());
        self.request(request, headers, sender)
    }

    #[inline]
    fn delete(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &Value,
    ) -> ClientResult<String> {
        let request = ureq::delete(url);
        let sender = |req: Request| req.send_json(payload.clone());
        self.request(request, headers, sender)
    }
}
