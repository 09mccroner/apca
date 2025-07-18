// Copyright (C) 2019-2023 The apca Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use url::Url;

use tokio::net::TcpStream;

use tracing::debug;
use tracing::span;
use tracing::trace;
use tracing::Level;
use tracing_futures::Instrument;

use tungstenite::connect_async;
use tungstenite::MaybeTlsStream;
use tungstenite::WebSocketStream;

use websocket_util::wrap::Wrapper;

use crate::Error;

/// A custom [`Result`]-style type that we can implement a foreign trait
/// on.
#[derive(Debug)]
#[doc(hidden)]
pub enum MessageResult<T, E> {
  /// The success value.
  Ok(T),
  /// The error value.
  Err(E),
}

impl<T, E> From<Result<T, E>> for MessageResult<T, E> {
  #[inline]
  fn from(result: Result<T, E>) -> Self {
    match result {
      Ok(t) => Self::Ok(t),
      Err(e) => Self::Err(e),
    }
  }
}

/// Internal function to connect to websocket server.
async fn connect_internal(url: &Url) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
  let span = span!(Level::DEBUG, "stream");

  async move {
    debug!(message = "connecting", url = display(url));

    // We just ignore the response & headers that are sent along after
    // the connection is made. Alpaca does not seem to be using them,
    // really.
    let (stream, response) = connect_async(url).await?;
    debug!("connection successful");
    trace!(response = debug(&response));

    Ok(stream)
  }
  .instrument(span)
  .await
}

/// Connect to a websocket server.
pub(crate) async fn connect(
  url: &Url,
) -> Result<Wrapper<WebSocketStream<MaybeTlsStream<TcpStream>>>, Error> {
  connect_internal(url)
    .await
    .map(|stream| Wrapper::builder().build(stream))
}

#[cfg(test)]
pub(crate) mod test {
  use super::*;

  use std::future::Future;

  use websocket_util::test::mock_server;
  use websocket_util::test::WebSocketStream;
  use websocket_util::tungstenite::Error as WebSocketError;

  use crate::subscribable::Subscribable;
  use crate::ApiInfo;

  /// The fake key-id we use.
  pub(crate) const KEY_ID: &str = "USER12345678";
  /// The fake secret we use.
  pub(crate) const SECRET: &str = "justletmein";

  /// Instantiate a dummy websocket server serving messages as per the
  /// provided function `f` and attempt to connect to it to stream
  /// messages.
  pub(crate) async fn mock_stream<S, F, R>(f: F) -> Result<(S::Stream, S::Subscription), Error>
  where
    S: Subscribable<Input = ApiInfo>,
    F: FnOnce(WebSocketStream) -> R + Send + Sync + 'static,
    R: Future<Output = Result<(), WebSocketError>> + Send + Sync + 'static,
  {
    let addr = mock_server(f).await;
    let stream_url = Url::parse(&format!("ws://{addr}")).unwrap();

    // We just set both the API stream URL and the data stream URL to
    // our websocket server. We don't know which one clients are trying
    // to mock, but currently it's only one or the other.
    let api_info = ApiInfo {
      api_base_url: Url::parse("http://example.com").unwrap(),
      api_stream_url: stream_url.clone(),
      data_base_url: Url::parse("http://example.com").unwrap(),
      data_stream_base_url: stream_url.clone(),
      key_id: KEY_ID.to_string(),
      secret: SECRET.to_string(),
    };

    S::connect(&api_info).await
  }
}
