use axum::Router;

/// Builder for composing Connect RPC services using an API similar to
/// `tonic`'s [`Routes`].
#[derive(Clone, Debug)]
pub struct Routes<S = ()> {
    router: Router<S>,
}

impl<S> Routes<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create an empty [`Routes`].
    pub fn new() -> Self {
        Self { router: Router::new() }
    }

    /// Create an empty [`Routes`] with the given state.
    pub fn with_state(state: S) -> Self {
        Self {
            router: Router::new().with_state(state),
        }
    }

    /// Add a generated Connect service to this builder.
    pub fn add_service<F>(mut self, svc: F) -> Self
    where
        F: FnOnce(Router<S>) -> crate::router::RpcRouter<S>,
    {
        self.router = svc(self.router);
        Self { router: self.router }
    }

    /// Convert this builder into an [`axum::Router`].
    pub fn into_router(self) -> Router<S> {
        self.router
    }

    /// Convert this builder directly into a [`tower::Service`].
    pub fn into_service<B>(self) -> axum::routing::RouterIntoService<B, S> {
        self.router.into_service()
    }
}
