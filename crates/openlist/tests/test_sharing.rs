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
                    "path": "/shared/docs",
                    "password": "abc123",
                    "expiration": "2025-12-31T23:59:59Z",
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
        .list(&PageReq { page: Some(1), per_page: Some(10) })
        .await
        .unwrap();

    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(1));
    let shares = data.content.unwrap();
    assert_eq!(shares.len(), 1);
    assert_eq!(shares[0].id.as_deref(), Some("share-001"));
    assert_eq!(shares[0].path.as_deref(), Some("/shared/docs"));
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
        .list(&PageReq { page: Some(1), per_page: Some(10) })
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
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "share-001".into()))
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
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "nonexistent".into()))
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
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": "share-new",
            "url": "https://example.com/s/share-new"
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
    assert_eq!(data.url.as_deref(), Some("https://example.com/s/share-new"));
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
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .sharing()
        .update(&Sharing {
            id: Some("share-001".into()),
            path: Some("/shared/docs".into()),
            url: None,
            password: Some("newpass".into()),
            expiration: Some("2025-12-31T23:59:59Z".into()),
            created_at: None,
            disabled: Some(false),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Delete Share ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_sharing_delete_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/share/delete")
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
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.sharing().disable("share-001").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
