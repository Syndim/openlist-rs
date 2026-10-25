mod common;

// ── Public Settings ──────────────────────────────────────────────────

#[tokio::test]
async fn test_public_settings_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/public/settings")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "version": "3.35.0",
            "site_title": "My OpenList",
            "pagination_type": "all",
            "default_page_size": "30",
            "search_index": "none"
        })))
        .create_async()
        .await;

    let resp = client.public().settings().await.unwrap();
    assert_eq!(resp.code, 200);
    let settings = resp.data.unwrap();
    assert_eq!(settings.get("version").map(|s| s.as_str()), Some("3.35.0"));
    assert_eq!(
        settings.get("site_title").map(|s| s.as_str()),
        Some("My OpenList")
    );
    mock.assert_async().await;
}

// ── Offline Download Tools ───────────────────────────────────────────

#[tokio::test]
async fn test_public_offline_download_tools_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/public/offline_download_tools")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            "aria2",
            "qbittorrent"
        ])))
        .create_async()
        .await;

    let resp = client.public().offline_download_tools().await.unwrap();
    let tools = resp.data.unwrap();
    let arr = tools.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].as_str().unwrap(), "aria2");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_public_offline_download_tools_empty() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/public/offline_download_tools")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([])))
        .create_async()
        .await;

    let resp = client.public().offline_download_tools().await.unwrap();
    let tools = resp.data.unwrap();
    assert!(tools.as_array().unwrap().is_empty());
    mock.assert_async().await;
}

// ── Archive Extensions ───────────────────────────────────────────────

#[tokio::test]
async fn test_public_archive_extensions_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/public/archive_extensions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            "zip", "rar", "7z", "tar", "tar.gz", "tar.bz2"
        ])))
        .create_async()
        .await;

    let resp = client.public().archive_extensions().await.unwrap();
    let exts = resp.data.unwrap();
    let arr = exts.as_array().unwrap();
    assert_eq!(arr.len(), 6);
    assert!(arr.iter().any(|v| v.as_str() == Some("zip")));
    assert!(arr.iter().any(|v| v.as_str() == Some("7z")));
    mock.assert_async().await;
}
