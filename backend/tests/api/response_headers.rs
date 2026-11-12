use crate::support::TestApp;
use reqwest::header::{
    CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};

async fn assert_security_headers(url: String) {
    let response = reqwest::get(&url).await.expect("request the page");
    let headers = response.headers();
    assert_eq!(headers[X_CONTENT_TYPE_OPTIONS], "nosniff", "{url}");
    assert_eq!(
        headers[REFERRER_POLICY], "strict-origin-when-cross-origin",
        "{url}"
    );
    assert_eq!(
        headers[CONTENT_SECURITY_POLICY], "frame-ancestors 'none'",
        "{url}"
    );
    assert_eq!(headers[X_FRAME_OPTIONS], "DENY", "{url}");
}

#[tokio::test]
async fn every_response_carries_the_security_headers() {
    let app = TestApp::spawn().await;

    assert_security_headers(format!("{}/api/v1/auth/session", app.base_url())).await;
    assert_security_headers(format!("{}/api/v1/monitor/health", app.base_url())).await;
    assert_security_headers(format!("{}/", app.base_url())).await;
    assert_security_headers(format!("{}/signin", app.base_url())).await;
}
