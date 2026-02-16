mod common;

use openlist::models::*;

// ── Get Current User ─────────────────────────────────────────────────

#[tokio::test]
async fn test_get_me_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": 1,
            "username": "admin",
            "password": "",
            "base_path": "/",
            "role": 2,
            "disabled": false,
            "permission": 29183,
            "sso_id": "",
            "otp": false
        })))
        .create_async()
        .await;

    let resp = client.user().me().await.unwrap();
    let user = resp.data.unwrap();
    assert_eq!(user.id, Some(1));
    assert_eq!(user.username.as_deref(), Some("admin"));
    assert_eq!(user.role, Some(2));
    assert_eq!(user.permission, Some(29183));
    assert_eq!(user.otp, Some(false));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_me_unauthorized() {
    let (mut server, _) = common::setup().await;
    let no_auth = openlist::OpenListClient::new(server.url());
    let mock = server
        .mock("GET", "/api/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(401, "Unauthorized"))
        .create_async()
        .await;

    let result = no_auth.user().me().await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Update Current User ──────────────────────────────────────────────

#[tokio::test]
async fn test_update_me_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/me/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .user()
        .update_me(&UpdateCurrentUserRequest {
            password: Some("newpass".into()),
            old_password: Some("oldpass".into()),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_me_wrong_old_password() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/me/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Invalid request or wrong old password"))
        .create_async()
        .await;

    let result = client
        .user()
        .update_me(&UpdateCurrentUserRequest {
            password: Some("newpass".into()),
            old_password: Some("wrongold".into()),
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── SSH Keys ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_ssh_keys_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/me/sshkey/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"id": 1, "title": "my-key", "key": "ssh-rsa AAAA...", "user_id": 1},
            {"id": 2, "title": "work-key", "key": "ssh-ed25519 AAAA...", "user_id": 1}
        ])))
        .create_async()
        .await;

    let resp = client.user().list_ssh_keys().await.unwrap();
    let keys = resp.data.unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].title.as_deref(), Some("my-key"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_ssh_keys_empty() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/me/sshkey/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([])))
        .create_async()
        .await;

    let resp = client.user().list_ssh_keys().await.unwrap();
    assert_eq!(resp.data.unwrap().len(), 0);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_add_ssh_key_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/me/sshkey/add")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .user()
        .add_ssh_key(&AddSshKeyRequest {
            title: "new-key".into(),
            key: "ssh-rsa AAAAB3NzaC1yc2E...".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_ssh_key_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/me/sshkey/delete")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .user()
        .delete_ssh_key(&DeleteSshKeyRequest { id: 1 })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
