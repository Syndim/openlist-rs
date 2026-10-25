use mockito::ServerGuard;
use openlist::OpenListClient;

pub async fn setup() -> (ServerGuard, OpenListClient) {
    let server = mockito::Server::new_async().await;
    let client = OpenListClient::new(server.url()).with_token("test-token");
    (server, client)
}

pub fn success_json(data: serde_json::Value) -> String {
    serde_json::json!({
        "code": 200,
        "message": "success",
        "data": data
    })
    .to_string()
}

/// Shared by the mock-based suites; not every test binary uses it.
#[allow(dead_code)]
pub fn error_json(code: i32, message: &str) -> String {
    serde_json::json!({
        "code": code,
        "message": message,
        "data": null
    })
    .to_string()
}
