//! Offline end-to-end tests for the global `--verbose` flag.
//!
//! These run the actual CLI binary against a mockito server, so they verify
//! that the flag is wired through argument parsing and into the SDK.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const ME_BODY: &str = r#"{"code":200,"message":"success","data":{"id":1,"username":"admin","password":"server-secret"}}"#;
const LOGIN_BODY: &str = r#"{"code":200,"message":"success","data":{"token":"server-token"}}"#;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_openlist"))
}

fn run(args: &[&str]) -> Output {
    cli().args(args).output().expect("failed to run openlist")
}

struct TempHome(PathBuf);

impl TempHome {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "openlist-cli-verbose-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("failed to create temporary home");
        Self(path)
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn verbose_flag_logs_request_and_response() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(ME_BODY)
        .create_async()
        .await;

    let url = server.url();
    let output = run(&["--server", &url, "--token", "client-secret", "-v", "me"]);

    assert!(
        output.status.success(),
        "openlist failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("> GET"), "missing request log:\n{stderr}");
    assert!(stderr.contains("/api/me"), "missing request URL:\n{stderr}");
    assert!(stderr.contains("< 200"), "missing response log:\n{stderr}");
    assert!(
        stderr.contains(r#""password": "<redacted>""#),
        "response body was not redacted:\n{stderr}"
    );
    assert!(
        !stderr.contains("client-secret"),
        "authorization header leaked:\n{stderr}"
    );
    assert!(
        !stderr.contains("server-secret"),
        "response secret leaked:\n{stderr}"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn verbose_flag_logs_and_redacts_login_request_body() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/auth/login")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LOGIN_BODY)
        .create_async()
        .await;

    let home = TempHome::new();
    let url = server.url();
    let output = cli()
        .args([
            "--server", &url, "-v", "login", "-u", "admin", "-p", "hunter2",
        ])
        .env("HOME", &home.0)
        .env("XDG_CONFIG_HOME", home.0.join(".config"))
        .env("APPDATA", &home.0)
        .output()
        .expect("failed to run openlist");

    assert!(
        output.status.success(),
        "openlist failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("> POST"), "missing request log:\n{stderr}");
    assert!(
        stderr.contains(r#""password": "<redacted>""#),
        "request body was not redacted:\n{stderr}"
    );
    assert!(
        stderr.contains(r#""token": "<redacted>""#),
        "response token was not redacted:\n{stderr}"
    );
    assert!(
        !stderr.contains("hunter2"),
        "request password leaked:\n{stderr}"
    );
    assert!(
        !stderr.contains("server-token"),
        "response token leaked:\n{stderr}"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn verbose_logging_is_silent_without_the_flag() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(ME_BODY)
        .create_async()
        .await;

    let url = server.url();
    let output = run(&["--server", &url, "--token", "client-secret", "me"]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("> GET"),
        "unexpected request log:\n{stderr}"
    );
    assert!(
        !stderr.contains("authorization"),
        "unexpected header log:\n{stderr}"
    );

    mock.assert_async().await;
}
