mod common;

use openlist::models::*;

// ── List Shares ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_list_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {
                    "id": "share-001",
                    "files": ["/shared/docs"],
                    "pwd": "abc123",
                    "expires": "2025-12-31T23:59:59Z",
                    "created_at": "2025-10-20T10:00:00Z",
                    "disabled": false
                }
            ],
            "total": 1
        })))
        .create_async()
        .await;

    let resp = client
        .sharing()
        .list(&PageReq {
            page: Some(1),
            per_page: Some(10),
        })
        .await
        .unwrap();

    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(1));
    let shares = data.content.unwrap();
    assert_eq!(shares.len(), 1);
    assert_eq!(shares[0].id.as_deref(), Some("share-001"));
    assert_eq!(
        shares[0].files.as_deref(),
        Some(["/shared/docs".to_string()].as_slice())
    );
    assert_eq!(shares[0].password.as_deref(), Some("abc123"));
    assert_eq!(shares[0].disabled, Some(false));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_sharing_list_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Forbidden"))
        .create_async()
        .await;

    let result = client
        .sharing()
        .list(&PageReq {
            page: Some(1),
            per_page: Some(10),
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Get Share ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_get_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/share/get")
        .match_query(mockito::Matcher::UrlEncoded(
            "id".into(),
            "share-001".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": "share-001",
            "path": "/shared/docs",
            "password": "",
            "expiration": "",
            "created_at": "2025-10-20T10:00:00Z",
            "disabled": false
        })))
        .create_async()
        .await;

    let resp = client.sharing().get("share-001").await.unwrap();
    let share = resp.data.unwrap();
    assert_eq!(share.id.as_deref(), Some("share-001"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_sharing_get_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/share/get")
        .match_query(mockito::Matcher::UrlEncoded(
            "id".into(),
            "nonexistent".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "Share not found"))
        .create_async()
        .await;

    let result = client.sharing().get("nonexistent").await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Create Share ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_create_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/create")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "files": ["/my/files"],
            "pwd": "secret",
            "expires": "2025-12-31T23:59:59Z"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": "share-new",
            "files": ["/my/files"],
            "pwd": "secret",
            "expires": "2025-12-31T23:59:59Z"
        })))
        .create_async()
        .await;

    let resp = client
        .sharing()
        .create(&CreateSharingRequest {
            paths: vec!["/my/files".into()],
            password: Some("secret".into()),
            expiration: Some("2025-12-31T23:59:59Z".into()),
        })
        .await
        .unwrap();

    let data = resp.data.unwrap();
    assert_eq!(data.id.as_deref(), Some("share-new"));
    assert_eq!(
        data.files.as_deref(),
        Some(["/my/files".to_string()].as_slice())
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn test_sharing_create_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Insufficient permissions"))
        .create_async()
        .await;

    let result = client
        .sharing()
        .create(&CreateSharingRequest {
            paths: vec!["/restricted".into()],
            password: None,
            expiration: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Update Share ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_update_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/update")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "id": "share-001",
            "files": ["/shared/docs"],
            "pwd": "newpass",
            "disabled": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": "share-001",
            "files": ["/shared/docs"],
            "pwd": "newpass",
            "disabled": true
        })))
        .create_async()
        .await;

    let resp = client
        .sharing()
        .update(&UpdateSharingRequest {
            id: "share-001".into(),
            files: vec!["/shared/docs".into()],
            pwd: Some("newpass".into()),
            expires: None,
            disabled: Some(true),
        })
        .await
        .unwrap();

    assert_eq!(resp.data.unwrap().disabled, Some(true));
    mock.assert_async().await;
}

// ── Delete Share ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_delete_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/delete")
        .match_query(mockito::Matcher::UrlEncoded(
            "id".into(),
            "share-001".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.sharing().delete("share-001").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Enable Share ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_enable_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/enable")
        .match_query(mockito::Matcher::UrlEncoded(
            "id".into(),
            "share-001".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.sharing().enable("share-001").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Disable Share ────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_disable_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/disable")
        .match_query(mockito::Matcher::UrlEncoded(
            "id".into(),
            "share-001".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.sharing().disable("share-001").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
