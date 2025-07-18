// Copyright (C) 2019-2024 The apca Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::v2::position::Position;
use crate::Str;

Endpoint! {
  /// The representation of a GET request to the /v2/positions endpoint.
  pub List(()),
  Ok => Vec<Position>, [
    /// The list of positions was retrieved successfully.
    /* 200 */ OK,
  ],
  Err => ListError, []

  #[inline]
  fn path(_input: &Self::Input) -> Str {
    "/v2/positions".into()
  }
}

// TODO: There is the possibility to issue a DELETE against the
//       /v2/positions endpoint in order to liquidate all open
//       positions, which may be interesting to use. However, that
//       requires support for multi-status HTTP responses.

#[cfg(test)]
mod tests {
  use super::*;

  use test_log::test;

  use crate::api_info::ApiInfo;
  use crate::Client;

  #[test(tokio::test)]
  async fn list_positions() {
    // We can't do much here except check that the request is not
    // reporting any errors.
    let api_info = ApiInfo::from_env().unwrap();
    let client = Client::new(api_info);
    let _ = client.issue::<List>(&()).await.unwrap();
  }
}
