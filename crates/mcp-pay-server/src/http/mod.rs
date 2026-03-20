pub mod middleware;
pub mod well_known;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::payment::X402Verifier;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub verifier: Arc<X402Verifier>,
}

/// Create the HTTP router for the server.
pub fn create_router(config: Config) -> Router {
    let verifier = Arc::new(X402Verifier::new(&config));

    let state = AppState {
        config,
        verifier,
    };

    // CORS configuration for browser-based clients
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Well-known endpoints
        .route("/.well-known/mcp-pay.json", get(well_known::mcp_pay_json))
        // API endpoints (payment-gated)
        .route("/api/weather", get(middleware::weather_current))
        .route("/api/forecast", get(middleware::weather_forecast))
        // Health check
        .route("/health", get(health))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let config = Config::test_config();
        let app = create_router(config);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
