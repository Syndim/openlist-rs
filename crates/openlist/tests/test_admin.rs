mod common;

use openlist::models::*;

// ── List Users ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_users_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/user/list")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            mockito::Matcher::UrlEncoded("per_page".into(), "30".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {"id": 1, "username": "admin", "role": 2, "permission": 29183, "disabled": false, "otp": false, "base_path": "/", "password": "", "sso_id": ""},
                {"id": 2, "username": "guest", "role": 1, "permission": 0, "disabled": false, "otp": false, "base_path": "/", "password": "", "sso_id": ""}
            ],
            "total": 2
        })))
        .create_async()
        .await;

    let resp = client.admin().list_users(Some(1), Some(30)).await.unwrap();
    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(2));
    assert_eq!(data.content.as_ref().unwrap().len(), 2);
    assert_eq!(
        data.content.as_ref().unwrap()[0].username.as_deref(),
        Some("admin")
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_list_users_forbidden() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/user/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(403, "Forbidden - Admin role required"))
        .create_async()
        .await;

    let result = client.admin().list_users(None, None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("403"));
    mock.assert_async().await;
}

// ── Get User ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_get_user_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/user/get")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": 1, "username": "admin", "role": 2, "permission": 29183,
            "disabled": false, "otp": false, "base_path": "/", "password": "", "sso_id": ""
        })))
        .create_async()
        .await;

    let resp = client.admin().get_user(1).await.unwrap();
    assert_eq!(resp.data.unwrap().username.as_deref(), Some("admin"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_user_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/user/get")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "999".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "User not found"))
        .create_async()
        .await;

    let result = client.admin().get_user(999).await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Create User ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_create_user_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .create_user(&CreateUserRequest {
            username: "newuser".into(),
            password: "password123".into(),
            base_path: Some("/".into()),
            role: Some(0),
            permission: Some(0),
            disabled: Some(false),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_create_user_duplicate() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(
            400,
            "Invalid request or username already exists",
        ))
        .create_async()
        .await;

    let result = client
        .admin()
        .create_user(&CreateUserRequest {
            username: "admin".into(),
            password: "pass".into(),
            base_path: None,
            role: None,
            permission: None,
            disabled: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Update User ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_update_user_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .update_user(&User {
            id: Some(2),
            username: Some("updated_user".into()),
            password: None,
            base_path: Some("/data".into()),
            role: Some(0),
            disabled: Some(false),
            permission: Some(255),
            sso_id: None,
            otp: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_update_user_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "User not found"))
        .create_async()
        .await;

    let result = client
        .admin()
        .update_user(&User {
            id: Some(999),
            username: None,
            password: None,
            base_path: None,
            role: None,
            disabled: None,
            permission: None,
            sso_id: None,
            otp: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Delete User ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_delete_user_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/delete")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().delete_user(2).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_delete_user_cannot_delete_admin() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/delete")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Cannot delete admin user"))
        .create_async()
        .await;

    let result = client.admin().delete_user(1).await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Cancel 2FA / Clear Cache ─────────────────────────────────────────

#[tokio::test]
async fn test_admin_cancel_2fa_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/cancel_2fa")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().cancel_2fa(2).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_del_user_cache_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/del_cache")
        .match_query(mockito::Matcher::UrlEncoded(
            "username".into(),
            "alice".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().del_user_cache("alice").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Admin SSH Keys ───────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_user_ssh_keys() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/user/sshkey/list")
        .match_query(mockito::Matcher::UrlEncoded("user_id".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"id": 1, "title": "key1", "key": "ssh-rsa AAA...", "user_id": 2}
        ])))
        .create_async()
        .await;

    let resp = client.admin().list_user_ssh_keys(2).await.unwrap();
    assert_eq!(resp.data.unwrap().len(), 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_delete_user_ssh_key() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/user/sshkey/delete")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .delete_user_ssh_key(&DeleteSshKeyRequest { id: 1 })
        .await
        .unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Storages ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_storages_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/storage/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "content": [
                {"id": 1, "mount_path": "/local", "driver": "Local", "status": "work", "disabled": false}
            ],
            "total": 1
        })))
        .create_async()
        .await;

    let resp = client.admin().list_storages(None, None).await.unwrap();
    let data = resp.data.unwrap();
    assert_eq!(data.total, Some(1));
    assert_eq!(
        data.content.as_ref().unwrap()[0].driver.as_deref(),
        Some("Local")
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_storage_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/storage/get")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": 1, "mount_path": "/local", "driver": "Local", "status": "work",
            "cache_expiration": 30, "order": 0, "disabled": false, "web_proxy": false,
            "webdav_policy": "native_proxy", "order_by": "name", "order_direction": "asc"
        })))
        .create_async()
        .await;

    let resp = client.admin().get_storage(1).await.unwrap();
    let s = resp.data.unwrap();
    assert_eq!(s.mount_path.as_deref(), Some("/local"));
    assert_eq!(s.status.as_deref(), Some("work"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_storage_not_found() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/storage/get")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "999".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(404, "Storage not found"))
        .create_async()
        .await;

    let result = client.admin().get_storage(999).await;
    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_create_storage_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .create_storage(&Storage {
            id: None,
            mount_path: Some("/new".into()),
            driver: Some("Local".into()),
            order: Some(0),
            cache_expiration: Some(30),
            addition: Some(r#"{"root_folder_path":"/data"}"#.into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_create_storage_duplicate_mount() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(
            400,
            "Invalid configuration or mount path already exists",
        ))
        .create_async()
        .await;

    let result = client
        .admin()
        .create_storage(&Storage {
            mount_path: Some("/local".into()),
            driver: Some("Local".into()),
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_update_storage_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .update_storage(&Storage {
            id: Some(1),
            cache_expiration: Some(60),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_delete_storage_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/delete")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().delete_storage(1).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_enable_storage() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/enable")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().enable_storage(1).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_disable_storage() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/disable")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().disable_storage(1).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_load_all_storages() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/storage/load_all")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().load_all_storages().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Drivers ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_drivers() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/driver/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"name": "Local", "config": {"root_folder_path": {"type": "string"}}},
            {"name": "S3", "config": {"bucket": {"type": "string"}}}
        ])))
        .create_async()
        .await;

    let resp = client.admin().list_drivers().await.unwrap();
    let drivers = resp.data.unwrap();
    assert_eq!(drivers.len(), 2);
    assert_eq!(drivers[0].name.as_deref(), Some("Local"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_list_driver_names() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/driver/names")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            "Local", "S3", "WebDAV", "FTP"
        ])))
        .create_async()
        .await;

    let resp = client.admin().list_driver_names().await.unwrap();
    let names = resp.data.unwrap();
    assert!(names.contains(&"Local".to_string()));
    assert_eq!(names.len(), 4);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_driver_info() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/driver/info")
        .match_query(mockito::Matcher::UrlEncoded(
            "driver".into(),
            "Local".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "name": "Local",
            "config": {"root_folder_path": {"type": "string", "required": true}}
        })))
        .create_async()
        .await;

    let resp = client.admin().get_driver_info("Local").await.unwrap();
    assert_eq!(resp.data.unwrap().name.as_deref(), Some("Local"));
    mock.assert_async().await;
}

// ── Settings ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_settings() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/setting/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"key": "site_title", "value": "OpenList", "type": "string", "group": 0, "flag": 0},
            {"key": "logo", "value": "", "type": "string", "group": 0, "flag": 0}
        ])))
        .create_async()
        .await;

    let resp = client.admin().list_settings().await.unwrap();
    let settings = resp.data.unwrap();
    assert_eq!(settings.len(), 2);
    assert_eq!(settings[0].key.as_deref(), Some("site_title"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_setting() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/setting/get")
        .match_query(mockito::Matcher::UrlEncoded(
            "key".into(),
            "site_title".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "key": "site_title", "value": "OpenList", "type": "string"
        })))
        .create_async()
        .await;

    let resp = client.admin().get_setting("site_title").await.unwrap();
    assert_eq!(resp.data.unwrap().value.as_deref(), Some("OpenList"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_save_settings() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/setting/save")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .save_settings(&[Setting {
            key: Some("site_title".into()),
            value: Some("My List".into()),
            setting_type: None,
            options: None,
            group: None,
            flag: None,
        }])
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_delete_setting() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/setting/delete")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().delete_setting("custom_key").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_reset_token() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/setting/reset_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().reset_token().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_default_settings() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/setting/default")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().default_settings().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Metas ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_metas() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/meta/list")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"id": 1, "path": "/protected", "password": "secret", "write": false}
        ])))
        .create_async()
        .await;

    let resp = client.admin().list_metas().await.unwrap();
    let metas = resp.data.unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].path.as_deref(), Some("/protected"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_meta() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/meta/get")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "id": 1, "path": "/protected", "password": "secret"
        })))
        .create_async()
        .await;

    let resp = client.admin().get_meta(1).await.unwrap();
    assert_eq!(resp.data.unwrap().password.as_deref(), Some("secret"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_create_meta() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/meta/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .create_meta(&Meta {
            id: None,
            path: Some("/new".into()),
            password: Some("pass".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_update_meta() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/meta/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .update_meta(&Meta {
            id: Some(1),
            password: Some("newpass".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_delete_meta() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/meta/delete")
        .match_query(mockito::Matcher::UrlEncoded("id".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().delete_meta(1).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── Search Index ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_build_index() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/index/build")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().build_index().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_update_index() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/index/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .admin()
        .update_index(&UpdateIndexRequest {
            paths: vec!["/local".into()],
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_stop_index() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/index/stop")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().stop_index().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_clear_index() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/index/clear")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().clear_index().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_index_progress() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/index/progress")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "done": false, "total": 1000, "current": 450
        })))
        .create_async()
        .await;

    let resp = client.admin().get_index_progress().await.unwrap();
    let prog = resp.data.unwrap();
    assert_eq!(prog.done, Some(false));
    assert_eq!(prog.total, Some(1000));
    assert_eq!(prog.current, Some(450));
    mock.assert_async().await;
}

// ── Scan ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_start_scan() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/scan/start")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().start_scan().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_stop_scan() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/admin/scan/stop")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.admin().stop_scan().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_admin_get_scan_progress() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/admin/scan/progress")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({"running": true})))
        .create_async()
        .await;

    let resp = client.admin().get_scan_progress().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
