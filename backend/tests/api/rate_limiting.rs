use crate::support::constants::{
    AI_BURST_FOR_TESTS, API_BURST_FOR_TESTS, FAST_REFILL_PER_MINUTE, FAST_REFILL_WAIT_MILLISECONDS,
    FIRST_CLIENT_ADDRESS, FORWARDED_FOR_HEADER, PLATFORM_MEMBER, RATE_LIMITED_CODE,
    RATE_LIMITED_MESSAGE_PREFIX, SECOND_CLIENT_ADDRESS, SIGN_IN_BURST_FOR_TESTS,
    SINGLE_SIGN_ON_PROVIDER, SLOWEST_REFILL_PER_MINUTE, TOO_MANY_ATTEMPTS_SIGN_IN_LOCATION,
    UNLIMITED_BURST_FOR_TESTS,
};
use crate::support::tenants::{create_user, sign_in};
use crate::support::{DeploymentFlags, TestApp, UserSession};
use reqwest::header::{LOCATION, RETRY_AFTER};
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};
use sopho::{RateLimitQuota, RateLimitSettings};

fn quota(burst_size: u32) -> RateLimitQuota {
    RateLimitQuota {
        requests_per_minute: SLOWEST_REFILL_PER_MINUTE,
        burst_size,
    }
}

fn rate_limited(is_proxy_client_address_trusted: bool) -> DeploymentFlags {
    DeploymentFlags {
        is_rate_limiting_on: true,
        is_proxy_client_address_trusted,
        rate_limits: Some(RateLimitSettings {
            sign_in: quota(SIGN_IN_BURST_FOR_TESTS),
            api: quota(API_BURST_FOR_TESTS),
            ai: quota(AI_BURST_FOR_TESTS),
        }),
        ..DeploymentFlags::default()
    }
}

fn only_sign_in_limited() -> DeploymentFlags {
    DeploymentFlags {
        rate_limits: Some(RateLimitSettings {
            sign_in: quota(SIGN_IN_BURST_FOR_TESTS),
            api: quota(UNLIMITED_BURST_FOR_TESTS),
            ai: quota(UNLIMITED_BURST_FOR_TESTS),
        }),
        ..rate_limited(true)
    }
}

async fn look_up_invitation_link(app: &TestApp, forwarded_for: Option<&str>) -> Response {
    let request = app
        .client
        .post(format!("{}/api/v1/invitation_link/lookup", app.base_url()))
        .json(&json!({ "token": "not-a-real-token" }));
    let request = match forwarded_for {
        Some(forwarded_for) => request.header(FORWARDED_FOR_HEADER, forwarded_for),
        None => request,
    };
    request.send().await.expect("look up the invitation link")
}

async fn use_up_the_sign_in_allowance(app: &TestApp, forwarded_for: &str) {
    for attempt in 0..SIGN_IN_BURST_FOR_TESTS {
        let response = look_up_invitation_link(app, Some(forwarded_for)).await;
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "attempt {attempt} was limited before the burst was used"
        );
    }
}

async fn assert_rate_limited(response: Response) {
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after_seconds: u64 = response.headers()[RETRY_AFTER]
        .to_str()
        .expect("retry-after is text")
        .parse()
        .expect("retry-after is a number of seconds");
    assert!(
        retry_after_seconds >= 1,
        "retry-after {retry_after_seconds}"
    );
    let body: Value = response.json().await.expect("the refusal is JSON");
    assert_eq!(body["error_code"], RATE_LIMITED_CODE, "{body}");
    let message = body["message"].as_str().expect("the refusal has a message");
    assert!(
        message.starts_with(RATE_LIMITED_MESSAGE_PREFIX),
        "{message}"
    );
}

async fn first_limited_request(session: &UserSession, path: &str, attempts: u32) -> Option<u32> {
    for attempt in 0..attempts {
        if session.get(path).await.status == StatusCode::TOO_MANY_REQUESTS {
            return Some(attempt);
        }
    }
    None
}

#[tokio::test]
async fn nothing_is_limited_while_rate_limiting_is_off() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    for _ in 0..40 {
        assert_ne!(
            look_up_invitation_link(&app, None).await.status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }
    assert_eq!(first_limited_request(&app, "/connection", 200).await, None);
}

#[tokio::test]
async fn sign_in_requests_beyond_the_burst_are_refused_for_that_client_address_only() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;
    use_up_the_sign_in_allowance(&app, FIRST_CLIENT_ADDRESS).await;

    assert_rate_limited(look_up_invitation_link(&app, Some(FIRST_CLIENT_ADDRESS)).await).await;
    let password_sign_in = app
        .client
        .post(format!("{}/api/v1/auth/session", app.base_url()))
        .header(FORWARDED_FOR_HEADER, FIRST_CLIENT_ADDRESS)
        .json(&json!({ "email": "nobody@example.test", "password": "wrong" }))
        .send()
        .await
        .expect("attempt a password sign-in");
    assert_rate_limited(password_sign_in).await;
    let reset_request = app
        .client
        .post(format!(
            "{}/api/v1/auth/password_reset_request",
            app.base_url()
        ))
        .header(FORWARDED_FOR_HEADER, FIRST_CLIENT_ADDRESS)
        .json(&json!({ "email": "nobody@example.test" }))
        .send()
        .await
        .expect("request a password reset");
    assert_rate_limited(reset_request).await;

    assert_ne!(
        look_up_invitation_link(&app, Some(SECOND_CLIENT_ADDRESS))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn a_limited_single_sign_on_start_and_callback_redirect_to_the_sign_in_page() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;
    use_up_the_sign_in_allowance(&app, FIRST_CLIENT_ADDRESS).await;
    let client = app.client_without_redirects();

    for single_sign_on_step in ["start", "callback?code=any&state=any"] {
        let response = client
            .get(format!(
                "{}/api/v1/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/{single_sign_on_step}",
                app.base_url()
            ))
            .header(FORWARDED_FOR_HEADER, FIRST_CLIENT_ADDRESS)
            .send()
            .await
            .expect("request the single sign-on step");
        assert!(
            response.status().is_redirection(),
            "{single_sign_on_step}: {}",
            response.status()
        );
        assert_eq!(
            response.headers()[LOCATION],
            TOO_MANY_ATTEMPTS_SIGN_IN_LOCATION,
            "{single_sign_on_step}"
        );
    }
}

#[tokio::test]
async fn token_refresh_session_check_health_and_pages_are_never_limited() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;
    use_up_the_sign_in_allowance(&app, FIRST_CLIENT_ADDRESS).await;

    for _ in 0..(SIGN_IN_BURST_FOR_TESTS * 4) {
        for (method, path) in [
            (reqwest::Method::POST, "/api/v1/auth/refresh"),
            (reqwest::Method::GET, "/api/v1/auth/session"),
            (reqwest::Method::DELETE, "/api/v1/auth/session"),
            (reqwest::Method::GET, "/api/v1/auth/sign_in_page_option"),
            (reqwest::Method::GET, "/api/v1/auth/oidc/provider"),
            (reqwest::Method::GET, "/api/v1/monitor/health"),
            (reqwest::Method::GET, "/signin"),
        ] {
            let response = app
                .client
                .request(method.clone(), format!("{}{path}", app.base_url()))
                .header(FORWARDED_FOR_HEADER, FIRST_CLIENT_ADDRESS)
                .send()
                .await
                .expect("send the unlimited request");
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "{method} {path}"
            );
            assert_ne!(
                response.status(),
                StatusCode::INTERNAL_SERVER_ERROR,
                "{method} {path}"
            );
        }
    }
}

#[tokio::test]
async fn an_untrusted_forwarded_address_gets_no_fresh_allowance() {
    let app = TestApp::spawn_with_deployment_flags(rate_limited(false)).await;

    for forged_address_suffix in 0..SIGN_IN_BURST_FOR_TESTS {
        let forged_address = format!("198.51.100.{forged_address_suffix}");
        assert_ne!(
            look_up_invitation_link(&app, Some(&forged_address))
                .await
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    assert_rate_limited(look_up_invitation_link(&app, Some("198.51.100.200")).await).await;
    assert_rate_limited(look_up_invitation_link(&app, None).await).await;
}

#[tokio::test]
async fn with_a_trusted_proxy_only_the_last_forwarded_entry_is_the_key() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;

    for forged_address_suffix in 0..SIGN_IN_BURST_FOR_TESTS {
        let forwarded_for = format!("198.51.100.{forged_address_suffix}, {FIRST_CLIENT_ADDRESS}");
        assert_ne!(
            look_up_invitation_link(&app, Some(&forwarded_for))
                .await
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    let forged_again = format!("198.51.100.200, {FIRST_CLIENT_ADDRESS}");
    assert_rate_limited(look_up_invitation_link(&app, Some(&forged_again)).await).await;
    let last_entry_differs = format!("{FIRST_CLIENT_ADDRESS}, {SECOND_CLIENT_ADDRESS}");
    assert_ne!(
        look_up_invitation_link(&app, Some(&last_entry_differs))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn with_a_trusted_proxy_a_request_without_a_forwarded_address_fails() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;

    for forwarded_for in [None, Some("unknown"), Some("")] {
        assert_eq!(
            look_up_invitation_link(&app, forwarded_for).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{forwarded_for:?}"
        );
    }
}

#[tokio::test]
async fn api_requests_are_limited_per_user_and_cover_the_account_routes() {
    let app = TestApp::spawn_with_deployment_flags(rate_limited(false)).await;
    app.login_as_admin().await;
    let other_user = create_user(&app, "other", PLATFORM_MEMBER).await;
    let other_session = sign_in(&app, &other_user).await;

    let first_limited = first_limited_request(&app, "/tenant", API_BURST_FOR_TESTS + 1).await;

    assert_eq!(first_limited, Some(API_BURST_FOR_TESTS));
    for account_path in ["/auth/me", "/auth/my_sign_in_method", "/connection"] {
        let limited = app.get(account_path).await;
        assert_eq!(
            limited.status,
            StatusCode::TOO_MANY_REQUESTS,
            "{account_path}: {}",
            limited.body
        );
        assert_eq!(limited.body["error_code"], RATE_LIMITED_CODE);
    }
    let other_users_request = other_session.get("/tenant").await;
    assert_eq!(
        other_users_request.status,
        StatusCode::OK,
        "{}",
        other_users_request.body
    );
}

#[tokio::test]
async fn a_signed_out_caller_gets_unauthorized_from_limited_routes() {
    let app = TestApp::spawn_with_deployment_flags(rate_limited(false)).await;

    for _ in 0..(API_BURST_FOR_TESTS + 2) {
        for path in ["/auth/me", "/auth/my_sign_in_method", "/tenant"] {
            let response = app.get(path).await;
            assert_eq!(
                response.status,
                StatusCode::UNAUTHORIZED,
                "{path}: {}",
                response.body
            );
        }
    }
    let ai_route = app
        .session(Some(app.tenant_id))
        .post("/ai_configuration/test", json!({}))
        .await;
    assert_eq!(
        ai_route.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        ai_route.body
    );
}

#[tokio::test]
async fn ai_requests_have_their_own_lower_limit() {
    let app = TestApp::spawn_with_deployment_flags(DeploymentFlags {
        rate_limits: Some(RateLimitSettings {
            sign_in: quota(UNLIMITED_BURST_FOR_TESTS),
            api: quota(UNLIMITED_BURST_FOR_TESTS),
            ai: quota(AI_BURST_FOR_TESTS),
        }),
        ..rate_limited(false)
    })
    .await;
    app.login_as_admin().await;
    let other_user = create_user(&app, "other", PLATFORM_MEMBER).await;
    let other_session = sign_in(&app, &other_user).await;
    let test_request = json!({ "provider": "anthropic", "api_key": "any-key" });

    for attempt in 0..AI_BURST_FOR_TESTS {
        let tested = app
            .post("/ai_configuration/test", test_request.clone())
            .await;
        assert_ne!(
            tested.status,
            StatusCode::TOO_MANY_REQUESTS,
            "attempt {attempt}: {}",
            tested.body
        );
    }

    let limited = app
        .post("/ai_configuration/test", test_request.clone())
        .await;
    assert_eq!(
        limited.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        limited.body
    );
    assert_eq!(limited.body["error_code"], RATE_LIMITED_CODE);
    let saved = app.put("/ai_configuration", test_request).await;
    assert_eq!(
        saved.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        saved.body
    );
    let unknown_identifier = uuid::Uuid::new_v4();
    for ai_path in [
        format!("/conversational_analytics/{unknown_identifier}/completion"),
        format!("/conversational_analytics/{unknown_identifier}/suggest-name"),
        format!("/ai_summary/dashboard/{unknown_identifier}"),
        format!("/ai_summary/dashboard/{unknown_identifier}/charts"),
        format!("/ai_summary/chart/{unknown_identifier}"),
    ] {
        let limited_ai_request = app.post(&ai_path, json!({})).await;
        assert_eq!(
            limited_ai_request.status,
            StatusCode::TOO_MANY_REQUESTS,
            "{ai_path}: {}",
            limited_ai_request.body
        );
    }
    let new_message = app
        .post(
            &format!("/conversational_analytics/{unknown_identifier}/messages"),
            json!({ "segments": [{ "type": "TEXT", "text": "Any question" }] }),
        )
        .await;
    assert_ne!(
        new_message.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        new_message.body
    );
    assert_eq!(app.get("/connection").await.status, StatusCode::OK);
    assert_eq!(other_session.get("/tenant").await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_allowance_refills_at_the_configured_rate() {
    let app = TestApp::spawn_with_deployment_flags(DeploymentFlags {
        rate_limits: Some(RateLimitSettings {
            sign_in: RateLimitQuota {
                requests_per_minute: FAST_REFILL_PER_MINUTE,
                burst_size: 1,
            },
            api: quota(UNLIMITED_BURST_FOR_TESTS),
            ai: quota(UNLIMITED_BURST_FOR_TESTS),
        }),
        ..rate_limited(true)
    })
    .await;

    assert_ne!(
        look_up_invitation_link(&app, Some(FIRST_CLIENT_ADDRESS))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_rate_limited(look_up_invitation_link(&app, Some(FIRST_CLIENT_ADDRESS)).await).await;

    tokio::time::sleep(std::time::Duration::from_millis(
        FAST_REFILL_WAIT_MILLISECONDS,
    ))
    .await;

    assert_ne!(
        look_up_invitation_link(&app, Some(FIRST_CLIENT_ADDRESS))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_rate_limited(look_up_invitation_link(&app, Some(FIRST_CLIENT_ADDRESS)).await).await;
}

#[tokio::test]
async fn addresses_in_one_ipv6_network_share_an_allowance() {
    let app = TestApp::spawn_with_deployment_flags(only_sign_in_limited()).await;

    for host_suffix in 0..SIGN_IN_BURST_FOR_TESTS {
        let rotated_address = format!("2001:db8:1:2::{host_suffix:x}");
        assert_ne!(
            look_up_invitation_link(&app, Some(&rotated_address))
                .await
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    assert_rate_limited(look_up_invitation_link(&app, Some("2001:db8:1:2:aaaa::1")).await).await;
    assert_ne!(
        look_up_invitation_link(&app, Some("2001:db8:1:3::1"))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}
