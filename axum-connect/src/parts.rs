use std::future::Future;

use axum::{
    extract::{
        connect_info::MockConnectInfo, ConnectInfo, FromRef, FromRequestParts, Query, State,
    },
    http::{self},
    Extension,
};
#[cfg(feature = "axum-extra")]
use axum_extra::extract::Host;
use prost::Message;
use serde::de::DeserializeOwned;

use crate::error::{RpcError, RpcErrorCode, RpcIntoError};

pub trait RpcFromRequestParts<T, S>: Sized
where
    T: Message,
    S: Send + Sync,
{
    /// If the extractor fails it'll use this "rejection" type. A rejection is
    /// a kind of error that can be converted into a response.
    type Rejection: RpcIntoError;

    /// Perform the extraction.
    fn rpc_from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}

#[cfg(feature = "axum-extra")]
impl<M, S> RpcFromRequestParts<M, S> for Host
where
    M: Message,
    S: Send + Sync,
{
    type Rejection = RpcError;

    async fn rpc_from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Host::from_request_parts(parts, state)
            .await
            .map_err(|e| (RpcErrorCode::Internal, e.to_string()).rpc_into_error())?)
    }
}

impl<M, S, T> RpcFromRequestParts<M, S> for Query<T>
where
    M: Message,
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = RpcError;

    async fn rpc_from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Query::from_request_parts(parts, state)
            .await
            .map_err(|e| (RpcErrorCode::Internal, e.to_string()).rpc_into_error())?)
    }
}

impl<M, S, T> RpcFromRequestParts<M, S> for ConnectInfo<T>
where
    M: Message,
    S: Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Rejection = RpcError;

    async fn rpc_from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match Extension::<Self>::from_request_parts(parts, state).await {
            Ok(Extension(connect_info)) => Ok(connect_info),
            Err(err) => match parts.extensions.get::<MockConnectInfo<T>>() {
                Some(MockConnectInfo(connect_info)) => Ok(Self(connect_info.clone())),
                None => Err((RpcErrorCode::Internal, err.to_string()).rpc_into_error()),
            },
        }
    }
}

impl<M, OuterState, InnerState> RpcFromRequestParts<M, OuterState> for State<InnerState>
where
    M: Message,
    InnerState: FromRef<OuterState>,
    OuterState: Send + Sync,
{
    type Rejection = RpcError;

    async fn rpc_from_request_parts(
        _parts: &mut http::request::Parts,
        state: &OuterState,
    ) -> Result<Self, Self::Rejection> {
        let inner_state = InnerState::from_ref(state);
        Ok(Self(inner_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Request,
        http::{HeaderName, HeaderValue, Method},
    };

    // Use pbjson_types Empty message as a simple test message
    use pbjson_types::Empty as TestMessage;

    // Custom extractor for user ID from x-user-id header
    #[derive(Debug)]
    struct ExtractUserId(String);

    impl<M, S> RpcFromRequestParts<M, S> for ExtractUserId
    where
        M: Message,
        S: Send + Sync,
    {
        type Rejection = RpcError;

        async fn rpc_from_request_parts(
            parts: &mut http::request::Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let user_id = parts
                .headers
                .get("x-user-id")
                .and_then(|value| value.to_str().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    (RpcErrorCode::InvalidArgument, "Missing x-user-id header").rpc_into_error()
                })?;

            Ok(ExtractUserId(user_id))
        }
    }

    #[tokio::test]
    async fn test_custom_extract_user_id() {
        // Test successful extraction
        let mut parts = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(
                HeaderName::from_static("x-user-id"),
                HeaderValue::from_static("user123"),
            )
            .body(())
            .unwrap()
            .into_parts()
            .0;

        let state = ();
        let result =
            <ExtractUserId as RpcFromRequestParts<TestMessage, ()>>::rpc_from_request_parts(
                &mut parts, &state,
            )
            .await;

        assert!(result.is_ok());
        if let Ok(ExtractUserId(user_id)) = result {
            assert_eq!(user_id, "user123");
        }

        // Test missing header
        let mut parts_no_header = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(())
            .unwrap()
            .into_parts()
            .0;

        let result_no_header =
            <ExtractUserId as RpcFromRequestParts<TestMessage, ()>>::rpc_from_request_parts(
                &mut parts_no_header,
                &state,
            )
            .await;

        assert!(result_no_header.is_err());
        if let Err(err) = result_no_header {
            assert!(matches!(err.code, RpcErrorCode::InvalidArgument));
            assert_eq!(err.message, "Missing x-user-id header");
        }
    }
}
