mod common;

use openlist::models::*;

// ── List Directory ───────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_list_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {
                    "id": "", "path": "/document.pdf", "name": "document.pdf",
                    "size": 1024000, "is_dir": false, "type": 4,
                    "modified": "2025-10-20T15:30:00+08:00",
                    "created": "2025-10-20T10:00:00+08:00",
                    "sign": "YBgnmykwCXUstXvNGtECaz_12gseXSL03cpqh5rTcGA=:0",
                    "thumb": "", "hashinfo": "null", "hash_info": null
                },
                {
                    "id": "", "path": "/photos", "name": "photos",
                    "size": 0, "is_dir": true, "type": 1,
                    "modified": "2025-10-20T12:00:00+08:00",
                    "created": "2025-10-19T08:00:00+08:00",
                    "sign": "", "thumb": "", "hashinfo": "null", "hash_info": null
                }
            ],
            "total": 14,
            "readme": "",
            "header": "",
            "write": true,
            "provider": "Local"
        })))
        .create_async()
        .await;

    let resp = client
        .fs()
        .list(&FsListRequest {
            path: Some("/".into()),
            password: None,
            refresh: None,
            page: Some(1),
            per_page: Some(30),
        })
        .await
        .unwrap();

    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(14));
    assert_eq!(data.provider.as_deref(), Some("Local"));
    assert_eq!(data.write, Some(true));
    let content = data.content.unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0].name.as_deref(), Some("document.pdf"));
    assert_eq!(content[0].size, Some(1024000));
    assert_eq!(content[0].is_dir, Some(false));
    assert_eq!(content[1].is_dir, Some(true));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_list_unauthorized() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(401, "Unauthorized"))
        .create_async()
        .await;

    let result = client
        .fs()
        .list(&FsListRequest {
            path: Some("/".into()),
            password: None,
            refresh: None,
            page: None,
            per_page: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_list_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Forbidden - insufficient permissions or wrong password"))
        .create_async()
        .await;

    let result = client
        .fs()
        .list(&FsListRequest {
            path: Some("/protected".into()),
            password: Some("wrong".into()),
            refresh: None,
            page: None,
            per_page: None,
        })
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("403"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_list_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "Path not found"))
        .create_async()
        .await;

    let result = client
        .fs()
        .list(&FsListRequest {
            path: Some("/nonexistent".into()),
            password: None,
            refresh: None,
            page: None,
            per_page: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Get File Info ────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_get_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/get")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": "", "path": "/document.pdf", "name": "document.pdf",
            "size": 1024000, "is_dir": false, "type": 4,
            "modified": "2025-10-20T15:30:00+08:00",
            "created": "2025-10-20T10:00:00+08:00",
            "sign": "abc123", "thumb": "", "hashinfo": "null",
            "hash_info": null,
            "mount_details": {
                "driver_name": "Local",
                "total_space": 1000000000000_i64,
                "free_space": 500000000000_i64
            }
        })))
        .create_async()
        .await;

    let resp = client
        .fs()
        .get(&FsGetRequest {
            path: "/document.pdf".into(),
            password: None,
        })
        .await
        .unwrap();

    let obj = resp.data.unwrap();
    assert_eq!(obj.name.as_deref(), Some("document.pdf"));
    assert_eq!(obj.size, Some(1024000));
    let details = obj.mount_details.unwrap();
    assert_eq!(details.driver_name.as_deref(), Some("Local"));
    assert_eq!(details.total_space, Some(1000000000000));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_get_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/get")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "File not found"))
        .create_async()
        .await;

    let result = client
        .fs()
        .get(&FsGetRequest {
            path: "/missing.txt".into(),
            password: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Search ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_search_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {"name": "document.pdf", "path": "/document.pdf", "is_dir": false, "size": 1024}
            ],
            "total": 1
        })))
        .create_async()
        .await;

    let resp = client
        .fs()
        .search(&FsSearchRequest {
            parent: "/".into(),
            keywords: "document".into(),
            scope: Some(1),
            page: Some(1),
            per_page: Some(30),
        })
        .await
        .unwrap();

    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(1));
    assert_eq!(data.content.as_ref().unwrap()[0].name.as_deref(), Some("document.pdf"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_search_index_not_enabled() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Search index not enabled"))
        .create_async()
        .await;

    let result = client
        .fs()
        .search(&FsSearchRequest {
            parent: "/".into(),
            keywords: "test".into(),
            scope: None,
            page: None,
            per_page: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Dirs ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_dirs_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/dirs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"name": "photos", "path": "/photos"},
            {"name": "docs", "path": "/docs"}
        ])))
        .create_async()
        .await;

    let resp = client
        .fs()
        .dirs(&FsDirsRequest {
            path: Some("/".into()),
            password: None,
            force_root: Some(false),
        })
        .await
        .unwrap();

    let dirs = resp.data.unwrap();
    assert_eq!(dirs.len(), 2);
    assert_eq!(dirs[0].name.as_deref(), Some("photos"));
    mock.assert_async().await;
}

// ── Mkdir ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_mkdir_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/mkdir")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .mkdir(&FsMkdirRequest {
            path: "/newfolder".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Rename ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_rename_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/rename")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .rename(&FsRenameRequest {
            path: "/oldname.txt".into(),
            name: "newname.txt".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Batch Rename ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_batch_rename_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/batch_rename")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .batch_rename(&FsBatchRenameRequest {
            src_dir: "/folder".into(),
            rename_objects: vec![
                RenameObject { src_name: "a.txt".into(), new_name: "b.txt".into() },
                RenameObject { src_name: "c.txt".into(), new_name: "d.txt".into() },
            ],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Regex Rename ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_regex_rename_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/regex_rename")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .regex_rename(&FsRegexRenameRequest {
            src_dir: "/folder".into(),
            src_name_regex: r"^(.+)\.bak$".into(),
            new_name: "$1".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Move ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_move_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/move")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .move_files(&FsMoveCopyRequest {
            src_dir: "/source".into(),
            dst_dir: "/destination".into(),
            names: vec!["file1.txt".into(), "file2.pdf".into()],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Recursive Move ───────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_recursive_move_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/recursive_move")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .recursive_move(&FsMoveCopyRequest {
            src_dir: "/source".into(),
            dst_dir: "/destination".into(),
            names: vec!["folder1".into()],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Copy ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_copy_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/copy")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .copy(&FsMoveCopyRequest {
            src_dir: "/source".into(),
            dst_dir: "/backup".into(),
            names: vec!["important.doc".into()],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Remove ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_remove_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/remove")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .remove(&FsRemoveRequest {
            dir: "/folder".into(),
            names: vec!["file1.txt".into(), "file2.pdf".into()],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Remove Empty Directory ───────────────────────────────────────────

#[tokio::test]
async fn test_fs_remove_empty_directory() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/remove_empty_directory")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .remove_empty_directory(&FsRemoveEmptyDirRequest {
            src_dir: "/cleanup".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Upload (stream) ──────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_put_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("PUT", "/api/fs/put")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let body = bytes::Bytes::from_static(b"file content here");
    let resp = client
        .fs()
        .put("/uploads/file.pdf", body, Some(false), None)
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_put_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("PUT", "/api/fs/put")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Insufficient permissions"))
        .create_async()
        .await;

    let body = bytes::Bytes::from_static(b"data");
    let result = client.fs().put("/restricted/file.bin", body, None, None).await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Other (provider-specific) ────────────────────────────────────────

#[tokio::test]
async fn test_fs_other_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/other")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({"url": "https://example.com/download"})))
        .create_async()
        .await;

    let resp = client
        .fs()
        .other(&FsOtherRequest {
            path: "/file.zip".into(),
            method: "video_preview".into(),
            password: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Offline Download ─────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_add_offline_download_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/add_offline_download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .add_offline_download(&AddOfflineDownloadRequest {
            path: "/downloads".into(),
            urls: vec!["https://example.com/file.zip".into()],
            tool: Some("aria2".into()),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_add_offline_download_not_configured() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/add_offline_download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Invalid request or offline download not configured"))
        .create_async()
        .await;

    let result = client
        .fs()
        .add_offline_download(&AddOfflineDownloadRequest {
            path: "/downloads".into(),
            urls: vec!["magnet:?xt=urn:btih:abc".into()],
            tool: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Archive ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fs_archive_decompress_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/archive/decompress")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .fs()
        .archive_decompress(&FsArchiveDecompressRequest {
            src_dir: "/archives".into(),
            name: "archive.zip".into(),
            dst_dir: "/extracted".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_archive_decompress_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/archive/decompress")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Insufficient permissions"))
        .create_async()
        .await;

    let result = client
        .fs()
        .archive_decompress(&FsArchiveDecompressRequest {
            src_dir: "/archives".into(),
            name: "archive.zip".into(),
            dst_dir: "/out".into(),
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_archive_meta_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/archive/meta")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "comment": "", "file_count": 5
        })))
        .create_async()
        .await;

    let resp = client
        .fs()
        .archive_meta(&FsArchiveMetaRequest {
            path: "/archives/test.zip".into(),
            password: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_fs_archive_list_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/fs/archive/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {"name": "readme.txt", "size": 256, "is_dir": false}
            ]
        })))
        .create_async()
        .await;

    let resp = client
        .fs()
        .archive_list(&FsArchiveListRequest {
            path: "/archives/test.zip".into(),
            password: None,
            inner_path: None,
            page: None,
            per_page: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
