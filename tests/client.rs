//! Client tests against a local mock of the CapMonster API (no network, no API key).
use std::time::Duration;

use capmonstercloud_client::requests::RecaptchaV2Request;
use capmonstercloud_client::{
    CapMonsterClient, ClientOptions, ClientProxy, Error, GetResultTimeouts, ProxyType, TaskStatus,
};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fast_client(server: &MockServer) -> CapMonsterClient {
    // 10 ms schedule instead of the real 1 s / 3 s / 180 s one.
    let options = ClientOptions::new("KEY")
        .with_service_url(server.uri())
        .with_timeouts_override(GetResultTimeouts::new(
            Duration::from_millis(10),
            Duration::ZERO,
            Duration::from_millis(10),
            Duration::from_millis(300),
        ));
    CapMonsterClient::new(options).unwrap()
}

fn request() -> RecaptchaV2Request {
    RecaptchaV2Request::new("https://example.com", "SITEKEY")
}

#[tokio::test]
async fn solve_captcha_creates_task_and_polls_until_ready() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .and(header_exists("user-agent"))
        .and(body_partial_json(json!({
            "clientKey": "KEY",
            "softId": 152,
            "task": {"type": "RecaptchaV2Task", "websiteURL": "https://example.com", "websiteKey": "SITEKEY"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "taskId": 42})))
        .expect(1)
        .mount(&server)
        .await;
    // First poll: still processing; second: ready.
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .and(body_partial_json(json!({"clientKey": "KEY", "taskId": 42})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "status": "processing"})),
        )
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errorId": 0, "status": "ready", "solution": {"gRecaptchaResponse": "TOKEN"}
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let solution = fast_client(&server)
        .solve_captcha(&request())
        .await
        .unwrap();
    assert_eq!(solution["gRecaptchaResponse"], "TOKEN");
}

#[tokio::test]
async fn create_task_api_error_is_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errorId": 1, "errorCode": "ERROR_KEY_DOES_NOT_EXIST", "errorDescription": "bad key"
        })))
        .mount(&server)
        .await;

    let err = fast_client(&server)
        .solve_captcha(&request())
        .await
        .unwrap_err();
    match err {
        Error::CreateTask { code, description } => {
            assert_eq!(code, "ERROR_KEY_DOES_NOT_EXIST");
            assert_eq!(description, "bad key");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn get_result_api_error_is_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "taskId": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errorId": 12, "errorCode": "ERROR_CAPTCHA_UNSOLVABLE", "errorDescription": "nope"
        })))
        .mount(&server)
        .await;

    let err = fast_client(&server)
        .solve_captcha(&request())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::GetResult { ref code, .. } if code == "ERROR_CAPTCHA_UNSOLVABLE"));
}

#[tokio::test]
async fn http_500_while_polling_means_still_processing() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let status = fast_client(&server).get_task_result(7).await.unwrap();
    assert_eq!(status, TaskStatus::Processing);
}

#[tokio::test]
async fn polling_gives_up_with_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "taskId": 1})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "status": "processing"})),
        )
        .mount(&server)
        .await;

    let err = fast_client(&server)
        .solve_captcha(&request())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Timeout(_)));
}

#[tokio::test]
async fn get_balance_returns_number() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/getBalance"))
        .and(body_partial_json(json!({"clientKey": "KEY"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"errorId": 0, "balance": 12.5})),
        )
        .mount(&server)
        .await;

    assert_eq!(fast_client(&server).get_balance().await.unwrap(), 12.5);
}

#[tokio::test]
async fn invalid_request_fails_before_any_http_call() {
    // No mock mounted: any HTTP call would return 404 and be reported as Error::Status.
    let server = MockServer::start().await;
    let bad =
        capmonstercloud_client::requests::ImageToTextRequest::new(vec![1]).with_threshold(200);
    let err = fast_client(&server).solve_captcha(&bad).await.unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

#[test]
fn socks_client_proxy_is_rejected() {
    let options = ClientOptions::new("KEY").with_client_proxy(ClientProxy::new(
        ProxyType::Socks5,
        "127.0.0.1",
        1080,
    ));
    assert!(matches!(
        CapMonsterClient::new(options),
        Err(Error::UnsupportedProxyType(_))
    ));
}
