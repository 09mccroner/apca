// Copyright (C) 2019-2024 The apca Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::str::from_utf8;

use http::Error as HttpError;
use http::StatusCode as HttpStatusCode;
use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use thiserror::Error;
use url::ParseError;
use websocket_util::tungstenite::Error as WebSocketError;

use crate::Str;

/// An error encountered while issuing a request.
#[derive(Debug, Error)]
pub enum RequestError<E> {
  /// An endpoint reported error.
  #[error("the endpoint reported an error")]
  Endpoint(#[source] E),
  /// An error reported by the `hyper` crate.
  #[error("the hyper crate reported an error")]
  Hyper(
    #[from]
    #[source]
    HyperError,
  ),
  /// An error reported by the `hyper-util` crate.
  #[error("the hyper-util crate reported an error")]
  HyperUtil(
    #[from]
    #[source]
    hyper_util::client::legacy::Error,
  ),
  /// An error reported while reading data.
  #[error("failed to read data")]
  Io(
    #[from]
    #[source]
    IoError,
  ),
}

#[derive(Clone, Debug, Error)]
pub struct HttpBody(Vec<u8>);

impl Display for HttpBody {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    match from_utf8(&self.0) {
      Ok(s) => fmt.write_str(s)?,
      Err(..) => write!(fmt, "{:?}", &self.0)?,
    }
    Ok(())
  }
}

/// The error type as used by this crate.
#[derive(Debug, Error)]
pub enum Error {
  /// An HTTP related error.
  #[error("encountered an HTTP related error")]
  Http(
    #[from]
    #[source]
    HttpError,
  ),
  /// We encountered an HTTP status code that either represents a
  /// failure or is not supported.
  #[error("encountered an unexpected HTTP status: {0}: {1}")]
  HttpStatus(HttpStatusCode, #[source] HttpBody),
  /// A JSON conversion error.
  #[error("a JSON conversion failed")]
  Json(
    #[from]
    #[source]
    JsonError,
  ),
  /// An error directly originating in this crate.
  #[error("{0}")]
  Str(Str),
  /// An URL parsing error.
  #[error("failed to parse the URL")]
  Url(
    #[from]
    #[source]
    ParseError,
  ),
  /// A websocket error.
  #[error("encountered a websocket related error")]
  WebSocket(
    #[from]
    #[source]
    WebSocketError,
  ),
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Check that we can serialize a [`Side`] object.
  #[test]
  fn formatting() {
    let body = HttpBody(vec![0, 159, 146, 150]);
    let err = Error::HttpStatus(HttpStatusCode::NOT_FOUND, body);
    assert_eq!(
      format!("{err}"),
      "encountered an unexpected HTTP status: 404 Not Found: [0, 159, 146, 150]"
    );

    let body = HttpBody("invalid".to_string().as_bytes().to_vec());
    let err = Error::HttpStatus(HttpStatusCode::NOT_FOUND, body);
    assert_eq!(
      format!("{err}"),
      "encountered an unexpected HTTP status: 404 Not Found: invalid"
    );
  }
}
