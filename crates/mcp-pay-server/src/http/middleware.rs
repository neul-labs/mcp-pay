//! Payment middleware and gated endpoints.
//!
//! This module implements the HTTP 402 payment flow for gated tools.

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::payment::{PaymentVerifier, PriceRequirement};
use crate::tools::weather;

use super::AppState;

/// Query parameters for weather endpoints.
#[derive(Debug, Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

/// Query parameters for forecast endpoint.
#[derive(Debug, Deserialize)]
pub struct ForecastQuery {
    pub city: String,
}

/// Get current weather (FREE).
///
/// This endpoint is not payment-gated.
pub async fn weather_current(
    State(_state): State<AppState>,
    Query(query): Query<WeatherQuery>,
) -> Response {
    let result = weather::get_current(&query.city).await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        result,
    )
        .into_response()
}

/// Get weather forecast (PAID).
///
/// This endpoint requires payment via x402.
pub async fn weather_forecast(
    State(state): State<AppState>,
    Query(query): Query<ForecastQuery>,
    headers: axum::http::HeaderMap,
) -> Response {
    let config = &state.config;

    // Check for payment header
    let payment_header = headers
        .get("X-PAYMENT")
        .and_then(|v| v.to_str().ok());

    let requirement = PriceRequirement::new(
        "get_forecast",
        &config.price_per_call,
        "USD",
    )
    .with_description("7-day weather forecast");

    match payment_header {
        Some(payment_payload) => {
            // Verify the payment
            match state.verifier.verify(payment_payload, &requirement).await {
                Ok(receipt) => {
                    // Payment verified, return the forecast
                    let result = weather::get_forecast(&query.city).await;
                    let receipt_header = receipt.to_header_value();

                    axum::http::Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("X-PAYMENT-RESPONSE", receipt_header)
                        .body(Body::from(result))
                        .unwrap()
                        .into_response()
                }
                Err(e) => {
                    // Payment invalid
                    let error_json = serde_json::json!({
                        "error": "payment_invalid",
                        "message": e.to_string()
                    });

                    axum::http::Response::builder()
                        .status(StatusCode::PAYMENT_REQUIRED)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("X-PAYMENT-ERROR", e.to_string())
                        .body(Body::from(serde_json::to_string(&error_json).unwrap()))
                        .unwrap()
                        .into_response()
                }
            }
        }
        None => {
            // No payment provided, return 402 with requirements
            let payment_required = state.verifier.create_payment_required(&requirement);
            let body = serde_json::to_string(&payment_required).unwrap();
            let header_value = payment_required.to_base64();

            axum::http::Response::builder()
                .status(StatusCode::PAYMENT_REQUIRED)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-PAYMENT-REQUIRED", header_value)
                .body(Body::from(body))
                .unwrap()
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::http::create_router;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_weather_current_free() {
        let config = Config::test_config();
        let app = create_router(config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/weather?city=London")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_forecast_requires_payment() {
        let config = Config::test_config();
        let app = create_router(config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/forecast?city=Tokyo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 402 Payment Required
        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);

        // Should include X-PAYMENT-REQUIRED header
        assert!(response.headers().contains_key("x-payment-required"));
    }
}
