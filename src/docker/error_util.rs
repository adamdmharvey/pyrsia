/*
   Copyright 2021 JFrog Ltd

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use log::debug;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Rejection, Reply};

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorMessage {
    code: RegistryErrorCode,
    message: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorMessages {
    errors: Vec<ErrorMessage>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum RegistryErrorCode {
    BlobUnknown,
    BlobDoesNotExist(String),
    ManifestUnknown,
    Unknown(String),
}

impl fmt::Display for RegistryErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match &self {
            RegistryErrorCode::BlobUnknown => "BLOB_UNKNOWN".to_string(),
            RegistryErrorCode::BlobDoesNotExist(hash) => format!("BLOB_DOES_NOT_EXIST({})", hash),
            RegistryErrorCode::ManifestUnknown => "MANIFEST_UNKNOWN".to_string(),
            RegistryErrorCode::Unknown(m) => format!("UNKNOWN({})", m),
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug, PartialEq)]
pub struct RegistryError {
    pub code: RegistryErrorCode,
}

impl From<anyhow::Error> for RegistryError {
    fn from(err: anyhow::Error) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl From<hex::FromHexError> for RegistryError {
    fn from(err: hex::FromHexError) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for RegistryError {
    fn from(err: reqwest::Error) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl From<Box<dyn Error>> for RegistryError {
    fn from(err: Box<dyn Error>) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl From<Box<dyn Error + Send>> for RegistryError {
    fn from(err: Box<dyn Error + Send>) -> RegistryError {
        RegistryError {
            code: RegistryErrorCode::Unknown(err.to_string()),
        }
    }
}

impl Reject for RegistryError {}

pub async fn custom_recover(err: Rejection) -> Result<impl Reply, Infallible> {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut error_message = ErrorMessage {
        code: RegistryErrorCode::Unknown("".to_string()),
        message: "".to_string(),
    };

    debug!("Rejection: {:?}", err);
    if let Some(e) = err.find::<RegistryError>() {
        match &e.code {
            RegistryErrorCode::BlobUnknown => {
                status_code = StatusCode::NOT_FOUND;
                error_message.code = RegistryErrorCode::BlobUnknown;
            }
            RegistryErrorCode::BlobDoesNotExist(hash) => {
                status_code = StatusCode::NOT_FOUND;
                error_message.code = RegistryErrorCode::BlobDoesNotExist(hash.to_string());
            }
            RegistryErrorCode::ManifestUnknown => {
                status_code = StatusCode::NOT_FOUND;
                error_message.code = RegistryErrorCode::ManifestUnknown;
            }
            RegistryErrorCode::Unknown(m) => {
                error_message.message = m.clone();
            }
        }
    } else if let Some(e) = err.find::<warp::reject::InvalidHeader>() {
        status_code = StatusCode::BAD_REQUEST;
        error_message.message = format!("{}", e);
    }

    debug!("ErrorMessage: {:?}", error_message);
    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorMessages {
            errors: vec![error_message],
        }),
        status_code,
    )
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::str;

    #[test]
    fn from_io_error() {
        let io_error_1 = io::Error::new(io::ErrorKind::Interrupted, "operation interrupted");
        let io_error_2 = io::Error::new(io::ErrorKind::Interrupted, "operation interrupted");

        let registry_error: RegistryError = io_error_1.into();
        assert_eq!(
            registry_error.code,
            RegistryErrorCode::Unknown(io_error_2.to_string())
        );
    }

    #[test]
    fn from_from_hex_error() {
        let from_hex_error = hex::FromHexError::OddLength;

        let registry_error: RegistryError = from_hex_error.into();
        assert_eq!(
            registry_error.code,
            RegistryErrorCode::Unknown(from_hex_error.to_string())
        );
    }

    #[test]
    fn from_anyhow_error() {
        let from_hex_error_1 = hex::FromHexError::OddLength;
        let anyhow_error_1: anyhow::Error = from_hex_error_1.into();

        let from_hex_error_2 = hex::FromHexError::OddLength;
        let anyhow_error_2: anyhow::Error = from_hex_error_2.into();

        let registry_error: RegistryError = anyhow_error_1.into();
        assert_eq!(
            registry_error.code,
            RegistryErrorCode::Unknown(anyhow_error_2.to_string())
        );
    }

    #[tokio::test]
    async fn recover_from_registry_error() {
        let registry_error = RegistryError {
            code: RegistryErrorCode::BlobUnknown,
        };

        let expected_body = serde_json::to_string(&ErrorMessages {
            errors: vec![ErrorMessage {
                code: RegistryErrorCode::BlobUnknown,
                message: "".to_string(),
            }],
        })
        .expect("Generating JSON body should not fail.");

        let response = custom_recover(registry_error.into())
            .await
            .expect("Reply should be created.")
            .into_response();
        let status = response.status();
        let actual_body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .expect("Response body to be converted to bytes");
        let actual_body_str = str::from_utf8(&actual_body_bytes)
            .map(str::to_owned)
            .expect("Response body to be converted to string");
        assert_eq!(status, 404);
        assert_eq!(actual_body_str, expected_body);
    }
}
