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

pub fn error_json(code: i32, message: &str) -> String {
    serde_json::json!({
        "code": code,
        "message": message,
        "data": null
    })
    .to_string()
}
