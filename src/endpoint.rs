// Copyright (C) 2019-2023 The apca Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;
use serde_json::Error as JsonError;
use serde_urlencoded::ser::Error as UrlEncodeError;

use thiserror::Error;

/// An error type comprising various conversion errors we may encounter.
#[derive(Debug, Error)]
pub enum ConversionError {
  /// A variant used when a JSON conversion failed.
  #[error("failed to convert from/to JSON")]
  Json(#[from] JsonError),
  /// A variant used when we fail to URL-encode a piece of data.
  #[error("failed to URL-encode data")]
  UrlEncode(#[from] UrlEncodeError),
}

/// An error as reported by API endpoints.
// Note that actually this type should probably be specific to the API
// version in question. However, at this point we only support v2, so we
// luck out here.
#[derive(Clone, Debug, Deserialize, Error, Eq, PartialEq)]
#[error("{message}")]
pub struct ApiError {
  /// A message as provided by Alpaca.
  #[serde(rename = "message")]
  pub message: String,
}

/// A macro used for defining the properties for a request to a
/// particular HTTP endpoint, without automated JSON parsing.
macro_rules! EndpointNoParse {
  ( $(#[$docs:meta])* $pub:vis $name:ident($in:ty),
    Ok => $out:ty, [$($(#[$ok_docs:meta])* $ok_status:ident,)*],
    Err => $err:ident, [$($(#[$err_docs:meta])* $err_status:ident => $variant:ident,)*]
    $($defs:tt)* ) => {

    EndpointDef! {
      $(#[$docs])* $pub $name($in),
      Ok => $out, [$($ok_status,)*],
      Err => $err, [
        // Every request can result in an authentication failure or fall
        // prey to the rate limit and so we include these variants into
        // all our error definitions.
        /// The request was not permitted.
        ///
        /// This can have a multitude of reasons, including invalid
        /// credentials or the (potentially implicit) request of SIP
        /// data through the data APIs when only an IEX subscription is
        /// available.
        /// Order submission/change failure (e.g., due to insufficient
        /// funds or time constraint violations) is also expressed this
        /// way.
        /* 403 */ FORBIDDEN => NotPermitted,
        /// The rate limit was exceeded, causing the request to be
        /// denied.
        /* 429 */ TOO_MANY_REQUESTS => RateLimitExceeded,
        $($(#[$err_docs])* $err_status => $variant,)*
      ],
      ConversionErr => crate::endpoint::ConversionError,
      ApiErr => crate::endpoint::ApiError,

      $($defs)*
    }
  };
}

/// A macro used for defining the properties for a request to a
/// particular HTTP endpoint.
macro_rules! Endpoint {
  ( $($input:tt)* ) => {
    EndpointNoParse! {
      $($input)*

      fn parse(body: &[u8]) -> Result<Self::Output, Self::ConversionError> {
        ::serde_json::from_slice::<Self::Output>(body).map_err(Self::ConversionError::from)
      }

      fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
        ::serde_json::from_slice::<Self::ApiError>(body).map_err(|_| body.to_vec())
      }
    }
  };
}
