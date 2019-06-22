use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use futures::Future;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, value::from_value, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub params: Vec<Value>,
    pub id: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub id: Value,
}

impl Response {
    // Extract the result from a response
    pub fn result<T>(&self) -> Result<T, ClientError>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        if let Some(ref e) = self.error {
            return Err(ClientError::Rpc(e.clone()));
        }
        match self.result {
            Some(ref res) => from_value(res.clone()).map_err(ClientError::Json),
            None => Err(ClientError::NoErrorOrResult),
        }
    }
}

// A handle to a remote JSONRPC server
pub struct JsonClient {
    endpoint: String,
    username: String,
    password: String,
    client: reqwest::r#async::Client,
    nonce: AtomicUsize,
}

impl JsonClient {
    pub fn new(endpoint: String, username: String, password: String) -> JsonClient {
        JsonClient {
            endpoint,
            username,
            password,
            client: reqwest::r#async::Client::new(),
            nonce: AtomicUsize::new(0), 
        }
    }

    /// Sends a request to a async client
    pub fn send_request(
        &self,
        request: &Request,
    ) -> Box<Future<Item = Response, Error = ClientError> + Send> {
        let mut request_builder = self.client.post(&self.endpoint);

        request_builder =
            request_builder.basic_auth(self.username.clone(), Some(self.password.clone()));
        let request_id = request.id.clone();
        let reqwest_response = request_builder
            .json(request)
            .send()
            .map_err(ClientError::from);

        // Parse response
        let response = reqwest_response.and_then(move |mut raw_resp| {
            let parsed_resp = raw_resp.json();
            parsed_resp
                .map_err(ClientError::from)
                .and_then(move |r: Response| {
                    if r.id != request_id {
                        futures::future::err(ClientError::NonceMismatch)
                    } else {
                        futures::future::ok(r)
                    }
                })
        });

        Box::new(response)
    }

    // Builds a request
    pub fn build_request(&self, method: String, params: Vec<Value>) -> Request {
        self.nonce.fetch_add(1, SeqCst);
        Request {
            method,
            params,
            id: json!(self.nonce.load(SeqCst)),
        }
    }
}

#[derive(Debug)]
pub enum ClientError {
    // Json decoding error.
    Json(serde_json::Error),
    // Client error
    Client(reqwest::Error),
    // Rpc error,
    Rpc(serde_json::Value),
    // Response has neither error nor result.
    NoErrorOrResult,
    // Response to a request did not have the expected nonce
    NonceMismatch,
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> ClientError {
        ClientError::Json(e)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> ClientError {
        ClientError::Client(e)
    }
}
