use crate::authentication::oidc::service::too_many_attempts_sign_in_path;
use crate::common::Configurations;
use crate::common::error_codes::ErrorCode;
use crate::feature_flag::constants::FeatureFlag;
use crate::rate_limit::constants::{
    LIMITER_CLEANUP_INTERVAL_SECONDS, RATE_LIMIT_KEY_UNAVAILABLE_MESSAGE,
    RATE_LIMIT_WINDOW_SECONDS, RETRY_AFTER_ROUNDING_SECONDS, SECOND_UNIT, SECONDS_UNIT,
    SINGLE_SECOND, TOO_MANY_REQUESTS_MESSAGE_PREFIX,
};
use crate::rate_limit::dto::RateLimitQuota;
use crate::rate_limit::key_extractor::{AuthenticatedUserKeyExtractor, ClientAddressKeyExtractor};
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::http::header::RETRY_AFTER;
use axum::http::{Response, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::routing::MethodRouter;
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};

type ClientAddressLimiter = Arc<GovernorConfig<ClientAddressKeyExtractor, NoOpMiddleware>>;
type AuthenticatedUserLimiter = Arc<GovernorConfig<AuthenticatedUserKeyExtractor, NoOpMiddleware>>;

#[derive(Clone)]
pub struct RateLimiters {
    sign_in: ClientAddressLimiter,
    api: AuthenticatedUserLimiter,
    ai: AuthenticatedUserLimiter,
}

fn build_limiter<SelectedKeyExtractor: KeyExtractor>(
    quota: RateLimitQuota,
    key_extractor: SelectedKeyExtractor,
) -> anyhow::Result<Arc<GovernorConfig<SelectedKeyExtractor, NoOpMiddleware>>> {
    let refill_period = Duration::from_secs(RATE_LIMIT_WINDOW_SECONDS) / quota.requests_per_minute;
    GovernorConfigBuilder::default()
        .period(refill_period)
        .burst_size(quota.burst_size)
        .key_extractor(key_extractor)
        .finish()
        .map(Arc::new)
        .ok_or_else(|| anyhow::anyhow!("a rate limit of {quota:?} cannot be enforced"))
}

pub fn build_rate_limiters(
    configurations: &Configurations,
) -> anyhow::Result<Option<RateLimiters>> {
    if !configurations
        .feature_flags
        .is_enabled(FeatureFlag::RateLimiting)
    {
        return Ok(None);
    }
    let is_proxy_trusted = configurations
        .feature_flags
        .is_enabled(FeatureFlag::TrustProxyClientAddress);
    Ok(Some(RateLimiters {
        sign_in: build_limiter(
            configurations.rate_limits.sign_in,
            ClientAddressKeyExtractor { is_proxy_trusted },
        )?,
        api: build_limiter(
            configurations.rate_limits.api,
            AuthenticatedUserKeyExtractor,
        )?,
        ai: build_limiter(configurations.rate_limits.ai, AuthenticatedUserKeyExtractor)?,
    }))
}

fn key_unavailable_response() -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": RATE_LIMIT_KEY_UNAVAILABLE_MESSAGE })),
    )
        .into_response()
}

fn too_many_requests_response(error: GovernorError) -> Response<Body> {
    let GovernorError::TooManyRequests { wait_time, .. } = error else {
        return key_unavailable_response();
    };
    let retry_after_seconds = wait_time.saturating_add(RETRY_AFTER_ROUNDING_SECONDS);
    let unit = match retry_after_seconds {
        SINGLE_SECOND => SECOND_UNIT,
        _ => SECONDS_UNIT,
    };
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(RETRY_AFTER, retry_after_seconds.to_string())],
        Json(serde_json::json!({
            "message":
                format!("{TOO_MANY_REQUESTS_MESSAGE_PREFIX} {retry_after_seconds} {unit}"),
            "error_code": ErrorCode::RateLimited.as_str(),
        })),
    )
        .into_response()
}

fn too_many_sign_in_navigations_response(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { .. } => {
            Redirect::to(&too_many_attempts_sign_in_path()).into_response()
        }
        GovernorError::UnableToExtractKey | GovernorError::Other { .. } => {
            key_unavailable_response()
        }
    }
}

pub fn limit_sign_in_requests<RouterState: Clone + Send + Sync + 'static>(
    rate_limiters: &Option<RateLimiters>,
    router: Router<RouterState>,
) -> Router<RouterState> {
    match rate_limiters {
        Some(rate_limiters) => router.route_layer(
            GovernorLayer::new(rate_limiters.sign_in.clone())
                .error_handler(too_many_requests_response),
        ),
        None => router,
    }
}

pub fn limit_sign_in_navigations<RouterState: Clone + Send + Sync + 'static>(
    rate_limiters: &Option<RateLimiters>,
    router: Router<RouterState>,
) -> Router<RouterState> {
    match rate_limiters {
        Some(rate_limiters) => router.route_layer(
            GovernorLayer::new(rate_limiters.sign_in.clone())
                .error_handler(too_many_sign_in_navigations_response),
        ),
        None => router,
    }
}

pub fn limit_api_requests<RouterState: Clone + Send + Sync + 'static>(
    rate_limiters: &Option<RateLimiters>,
    router: Router<RouterState>,
) -> Router<RouterState> {
    match rate_limiters {
        Some(rate_limiters) => router.route_layer(
            GovernorLayer::new(rate_limiters.api.clone()).error_handler(too_many_requests_response),
        ),
        None => router,
    }
}

pub fn limit_ai_requests<RouterState: Clone + Send + Sync + 'static>(
    rate_limiters: &Option<RateLimiters>,
    method_router: MethodRouter<RouterState>,
) -> MethodRouter<RouterState> {
    match rate_limiters {
        Some(rate_limiters) => method_router.route_layer(
            GovernorLayer::new(rate_limiters.ai.clone()).error_handler(too_many_requests_response),
        ),
        None => method_router,
    }
}

pub fn start_limiter_cleanup(rate_limiters: &Option<RateLimiters>) {
    let Some(rate_limiters) = rate_limiters.clone() else {
        return;
    };
    tokio::spawn(async move {
        let mut cleanup_interval =
            tokio::time::interval(Duration::from_secs(LIMITER_CLEANUP_INTERVAL_SECONDS));
        loop {
            cleanup_interval.tick().await;
            rate_limiters.sign_in.limiter().retain_recent();
            rate_limiters.sign_in.limiter().shrink_to_fit();
            rate_limiters.api.limiter().retain_recent();
            rate_limiters.api.limiter().shrink_to_fit();
            rate_limiters.ai.limiter().retain_recent();
            rate_limiters.ai.limiter().shrink_to_fit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::too_many_requests_response;
    use axum::http::StatusCode;
    use axum::http::header::RETRY_AFTER;
    use tower_governor::GovernorError;

    async fn refusal_for_wait(wait_time: u64) -> (String, serde_json::Value) {
        let response = too_many_requests_response(GovernorError::TooManyRequests {
            wait_time,
            headers: None,
        });
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let retry_after = response.headers()[RETRY_AFTER]
            .to_str()
            .unwrap()
            .to_string();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (retry_after, serde_json::from_slice(&body_bytes).unwrap())
    }

    #[tokio::test]
    async fn the_wait_is_rounded_up_and_never_zero() {
        let (retry_after, body) = refusal_for_wait(0).await;
        assert_eq!(retry_after, "1");
        assert_eq!(body["message"], "Too many requests. Try again in 1 second");
        assert_eq!(body["error_code"], "RATE_LIMITED");

        let (retry_after, body) = refusal_for_wait(5).await;
        assert_eq!(retry_after, "6");
        assert_eq!(body["message"], "Too many requests. Try again in 6 seconds");
    }

    #[tokio::test]
    async fn a_request_that_cannot_be_attributed_is_a_server_error() {
        let response = too_many_requests_response(GovernorError::UnableToExtractKey);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
