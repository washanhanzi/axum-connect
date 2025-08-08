//! Tower Layer and Service for handling Connect streams.
//!
//! This module provides `ConnectStreamLayer`, a Tower layer that is intended to
//! enable handling of Connect server streams.
//!
//! The core logic is in the `ConnectStream` response wrapper, which is the idiomatic
//! way to handle streaming responses in Axum.

use crate::error::ConnectError;
use axum::{
    body::{Body, Bytes},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
};
use futures::{Stream, TryStreamExt};
use prost::Message;
use serde::Serialize;
use tokio_util::io::StreamReader;
use tower::Layer;

/// A layer for Connect server streaming.
///
/// This layer is currently a no-op. The streaming logic is handled by the
/// `ConnectStream` response wrapper. This layer is included to satisfy the
/// refactoring plan and may be used for more advanced features in the future.
#[derive(Clone, Copy, Debug)]
pub struct ConnectStreamLayer;

impl<S> Layer<S> for ConnectStreamLayer {
    type Service = S;

    fn layer(&self, inner: S) -> Self::Service {
        inner
    }
}

/// A response wrapper for server-streaming handlers.
///
/// This wrapper takes a stream of messages and encodes them according to the
/// Connect protocol for server streams. It defaults to JSON encoding.
#[derive(Debug)]
pub struct ConnectStream<S> {
    stream: S,
}

impl<S> ConnectStream<S> {
    /// Create a new `ConnectStream` from a stream of messages.
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S, T> IntoResponse for ConnectStream<S>
where
    S: Stream<Item = Result<T, ConnectError>> + Send + 'static,
    T: Message + Serialize + Send + 'static,
{
    fn into_response(self) -> Response {
        let body_stream = self
            .stream
            .map_ok(|msg| {
                let mut buf = vec![0u8; 5];
                serde_json::to_writer(&mut buf, &msg).unwrap();
                let len = (buf.len() - 5) as u32;
                buf[1..5].copy_from_slice(&len.to_be_bytes());
                Bytes::from(buf)
            })
            .or_else(|err| {
                let mut buf = vec![0b0000_0010u8, 0, 0, 0, 0]; // End stream flag
                let json = serde_json::json!({ "error": err });
                serde_json::to_writer(&mut buf, &json).unwrap();
                let len = (buf.len() - 5) as u32;
                buf[1..5].copy_from_slice(&len.to_be_bytes());
                futures::future::ready(Ok(Bytes::from(buf)))
            });

        let reader = StreamReader::new(body_stream);
        let body = Body::from_async(reader);

        Response::builder()
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/connect+json"),
            )
            .body(body)
            .unwrap()
    }
}
