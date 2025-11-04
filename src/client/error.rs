use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

fn default_aliyun_rejection_field() -> String {
    "No content. If you see this please submit a bug issue (to the rust sdk repo).".to_string()
}

/// Represents an error response returned by the Aliyun server.
///
/// The response fields use PascalCase naming convention to match
/// the format returned by the Aliyun API and maintain consistency.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AliyunRejection {
    pub code: String,
    pub host_id: String,
    #[serde(default = "default_aliyun_rejection_field")]
    pub message: String,
    pub request_id: String,
    #[serde(default = "default_aliyun_rejection_field")]
    pub recommend: String,
}

#[derive(Serialize, Debug)]
pub enum RequestErrorKind {
    Timeout,
    Connect,
}

#[derive(Serialize, Error, Debug)]
pub enum OperationError {
    #[error("Aliyun rejected the request and returned an error")]
    Rejected(AliyunRejection),

    #[error("{}", .message)]
    RequestFailure {
        message: String,
        kind: RequestErrorKind,
        #[serde(skip)]
        source: reqwest::Error,
    },

    #[error("{}", .message)]
    InternalError {
        message: String,
        #[serde(skip)]
        source: Box<dyn std::error::Error>,
    },
}

impl From<reqwest::Error> for OperationError {
    fn from(source: reqwest::Error) -> Self {
        let message = format!("(reqwest) {:?}", source);
        let request_error_kind = if source.is_timeout() {
            Some(RequestErrorKind::Timeout)
        } else if source.is_connect() {
            Some(RequestErrorKind::Connect)
        } else {
            None
        };

        if let Some(kind) = request_error_kind {
            Self::RequestFailure {
                message,
                kind,
                source,
            }
        } else {
            Self::InternalError {
                message,
                source: Box::new(source),
            }
        }
    }
}

impl From<serde_json::Error> for OperationError {
    fn from(value: serde_json::Error) -> Self {
        let message = format!("(json (de)serialization error) {:?}", value);
        Self::InternalError {
            message,
            source: Box::new(value),
        }
    }
}
