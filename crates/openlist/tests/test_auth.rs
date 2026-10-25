mod common;

use openlist::models::*;

// ── Login ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test"
        })))
        .create_async()
        .await;

    let resp = client
        .auth()
        .login(&LoginRequest {
            username: "admin".into(),
            password: "my password".into(),
            otp_code: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    assert_eq!(resp.message, "success");
    let data = resp.data.unwrap();
    assert!(data.token.starts_with("eyJ"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Invalid request parameters"))
        .create_async()
        .await;

    let result = client
        .auth()
        .login(&LoginRequest {
            username: "admin".into(),
            password: "wrong".into(),
            otp_code: None,
        })
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("400"));
    assert!(err.to_string().contains("Invalid request parameters"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_login_with_otp() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "eyJhbGciOiJIUzI1NiJ9.otp_test"
        })))
        .create_async()
        .await;

    let resp = client
        .auth()
        .login(&LoginRequest {
            username: "admin".into(),
            password: "my password".into(),
            otp_code: Some("123456".into()),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    assert!(resp.data.unwrap().token.contains("otp_test"));
    mock.assert_async().await;
}

// ── Login Hash ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_hash_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login/hash")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "hash_token_123"
        })))
        .create_async()
        .await;

    let resp = client
        .auth()
        .login_hash(&LoginHashRequest {
            username: "admin".into(),
            password: "hashed_password_string".into(),
            otp_code: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.data.unwrap().token, "hash_token_123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_login_hash_failure() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login/hash")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "Authentication failed"))
        .create_async()
        .await;

    let result = client
        .auth()
        .login_hash(&LoginHashRequest {
            username: "admin".into(),
            password: "bad_hash".into(),
            otp_code: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── LDAP Login ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_ldap_login_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login/ldap")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "ldap_token_abc"
        })))
        .create_async()
        .await;

    let resp = client
        .auth()
        .login_ldap(&LdapLoginRequest {
            username: "ldapuser".into(),
            password: "ldappass".into(),
            otp_code: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.data.unwrap().token, "ldap_token_abc");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_ldap_login_failure() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/login/ldap")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(400, "LDAP authentication failed"))
        .create_async()
        .await;

    let result = client
        .auth()
        .login_ldap(&LdapLoginRequest {
            username: "bad".into(),
            password: "bad".into(),
            otp_code: None,
        })
        .await;

    assert!(result.is_err());
    mock.assert_async().await;
}

// ── Logout ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_logout_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/auth/logout")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.auth().logout().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_logout_unauthorized() {
    let (mut server, _) = common::setup().await;
    let no_auth_client = openlist::OpenListClient::new(server.url());
    let mock = server
        .mock("GET", "/api/auth/logout")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(401, "Unauthorized"))
        .create_async()
        .await;

    let result = no_auth_client.auth().logout().await;
    assert!(result.is_err());
    mock.assert_async().await;
}

// ── 2FA ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_generate_2fa_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/2fa/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "secret": "JBSWY3DPEHPK3PXP",
            "qr": "data:image/png;base64,iVBOR..."
        })))
        .create_async()
        .await;

    let resp = client.auth().generate_2fa().await.unwrap();
    let data = resp.data.unwrap();
    assert_eq!(data.secret.unwrap(), "JBSWY3DPEHPK3PXP");
    assert!(data.qr.unwrap().starts_with("data:image"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_generate_2fa_unauthorized() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/2fa/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::error_json(401, "Unauthorized"))
        .create_async()
        .await;

    let result = client.auth().generate_2fa().await;
    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_verify_2fa_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/auth/2fa/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client
        .auth()
        .verify_2fa(&Verify2FARequest {
            code: "123456".into(),
            secret: "JBSWY3DPEHPK3PXP".into(),
        })
        .await
        .unwrap();

    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── SSO ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_sso_login_redirect() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/auth/sso")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(
            "https://sso.example.com/authorize?..."
        )))
        .create_async()
        .await;

    let resp = client.auth().sso_login_redirect().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_sso_callback() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/auth/sso_callback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "sso_token"
        })))
        .create_async()
        .await;

    let resp = client.auth().sso_callback().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

// ── WebAuthn ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_begin_webauthn_login() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/authn/webauthn_begin_login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "challenge": "base64challenge",
            "timeout": 60000
        })))
        .create_async()
        .await;

    let resp = client.auth().begin_webauthn_login().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_finish_webauthn_login_success() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/authn/webauthn_finish_login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "token": "webauthn_token"
        })))
        .create_async()
        .await;

    let body = serde_json::json!({"id": "cred_id", "response": {}});
    let resp = client.auth().finish_webauthn_login(&body).await.unwrap();
    assert_eq!(resp.data.unwrap().token, "webauthn_token");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_begin_webauthn_registration() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/authn/webauthn_begin_registration")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!({
            "challenge": "reg_challenge"
        })))
        .create_async()
        .await;

    let resp = client.auth().begin_webauthn_registration().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_finish_webauthn_registration() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/authn/webauthn_finish_registration")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let body = serde_json::json!({"id": "new_cred", "response": {}});
    let resp = client
        .auth()
        .finish_webauthn_registration(&body)
        .await
        .unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_webauthn() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/authn/delete_authn")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let body = serde_json::json!({"id": "cred_to_delete"});
    let resp = client.auth().delete_webauthn(&body).await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_webauthn_credentials() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/authn/getcredentials")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            {"id": "cred1", "name": "My Key"}
        ])))
        .create_async()
        .await;

    let resp = client.auth().get_webauthn_credentials().await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}
