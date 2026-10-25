use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::error::{OpenListError, Result};
use crate::models::ApiResponse;

#[derive(Clone)]
pub struct OpenListClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) token: Option<String>,
    pub(crate) verbose: bool,
}

impl OpenListClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: None,
            verbose: false,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Enable or disable verbose request/response logging to stderr.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Execute a prepared request, logging it when verbose mode is enabled.
    pub(crate) async fn send(&self, builder: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = builder.build()?;
        if self.verbose {
            log_request(&request);
        }

        let response = self.http.execute(request).await?;

        if self.verbose {
            eprintln!("< {} {}", response.status(), response.url());
            log_headers("<", response.headers());
        }

        Ok(response)
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(crate) fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(ref token) = self.token
            && let Ok(val) = HeaderValue::from_str(token)
        {
            headers.insert(AUTHORIZATION, val);
        }
        headers
    }

    pub(crate) async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<ApiResponse<T>> {
        let status = resp.status();
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        // Read the body before parsing: when a proxy answered instead of the
        // API the body is not JSON, and the status plus those bytes are the evi-
        // dence that identifies the failing hop.
        let bytes = resp.bytes().await?;
        if self.verbose {
            log_body("<", &bytes, &content_type);
        }
        let api_resp: ApiResponse<T> = serde_json::from_slice(&bytes).map_err(|source| {
            crate::error::OpenListError::non_json_response(
                status.as_u16(),
                content_type,
                body_preview(&bytes),
                source,
            )
        })?;
        if api_resp.code != 200 {
            return Err(OpenListError::Api {
                code: api_resp.code,
                message: api_resp.message.clone(),
            });
        }
        Ok(api_resp)
    }

    // Auth helpers
    pub fn auth(&self) -> crate::api::auth::AuthApi<'_> {
        crate::api::auth::AuthApi { client: self }
    }

    pub fn user(&self) -> crate::api::user::UserApi<'_> {
        crate::api::user::UserApi { client: self }
    }

    pub fn admin(&self) -> crate::api::admin::AdminApi<'_> {
        crate::api::admin::AdminApi { client: self }
    }

    pub fn fs(&self) -> crate::api::fs::FsApi<'_> {
        crate::api::fs::FsApi { client: self }
    }

    pub fn public(&self) -> crate::api::public::PublicApi<'_> {
        crate::api::public::PublicApi { client: self }
    }

    pub fn sharing(&self) -> crate::api::sharing::SharingApi<'_> {
        crate::api::sharing::SharingApi { client: self }
    }

    pub fn task(&self) -> crate::api::task::TaskApi<'_> {
        crate::api::task::TaskApi { client: self }
    }
}

/// Extension trait that routes `reqwest` request builders through the client so
/// every operation can be logged consistently in verbose mode.
pub(crate) trait RequestBuilderExt {
    async fn send_with(self, client: &OpenListClient) -> Result<reqwest::Response>;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    async fn send_with(self, client: &OpenListClient) -> Result<reqwest::Response> {
        client.send(self).await
    }
}

fn log_request(request: &reqwest::Request) {
    eprintln!("> {} {}", request.method(), request.url());
    log_headers(">", request.headers());

    let Some(bytes) = request.body().and_then(|body| body.as_bytes()) else {
        return;
    };
    let content_type = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    log_body(">", bytes, content_type);
}

/// Render a short, single-line preview of a response body for error messages.
fn body_preview(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "<empty>".to_string();
    }
    const LIMIT: usize = 400;
    let end = bytes.len().min(LIMIT);
    let text = String::from_utf8_lossy(&bytes[..end]);
    let mut out = String::new();
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() || ch.is_control() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(ch);
    }
    if out.is_empty() {
        out.push_str("<binary body>");
    }
    if bytes.len() > LIMIT {
        out.push('…');
    }
    out
}

/// Percent-encode a remote path for the `File-Path` header, which OpenList
/// decodes with `url.PathUnescape`. Unreserved characters and `/` are left
/// alone so logs stay readable.
pub(crate) fn encode_path(path: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(path.len());
    for &byte in path.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    out
}

fn log_body(prefix: &str, bytes: &[u8], content_type: &str) {
    if bytes.is_empty() {
        return;
    }

    if content_type.starts_with("application/json") {
        match serde_json::from_slice::<serde_json::Value>(bytes) {
            Ok(mut value) => {
                redact_json(&mut value);
                let rendered = serde_json::to_string_pretty(&value)
                    .unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned());
                eprintln!("{prefix} {rendered}");
            }
            Err(_) => eprintln!("{prefix} {}", String::from_utf8_lossy(bytes)),
        }
    } else if content_type.starts_with("text/")
        || content_type.starts_with("application/x-www-form-urlencoded")
    {
        eprintln!("{prefix} {}", String::from_utf8_lossy(bytes));
    } else if content_type.is_empty() {
        match std::str::from_utf8(bytes) {
            Ok(text) => eprintln!("{prefix} {text}"),
            Err(_) => eprintln!("{prefix} <{} bytes>", bytes.len()),
        }
    } else {
        eprintln!("{prefix} <{} bytes>", bytes.len());
    }
}

fn log_headers(prefix: &str, headers: &HeaderMap) {
    for (name, value) in headers {
        let rendered = if is_sensitive_header(name.as_str()) {
            "<redacted>"
        } else {
            value.to_str().unwrap_or("<binary>")
        };
        eprintln!("{prefix} {}: {}", name, rendered);
    }
}

fn is_sensitive_header(name: &str) -> bool {
    matches!(
        name,
        "authorization" | "proxy-authorization" | "cookie" | "set-cookie"
    )
}

fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *value = serde_json::Value::String("<redacted>".to_string());
                } else {
                    redact_json(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(redact_json),
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    matches!(
        key,
        "password" | "old_password" | "new_password" | "token" | "otp_code" | "secret"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_toggle_controls_flag() {
        let mut client = OpenListClient::new("http://example.test");
        assert!(!client.is_verbose());

        client.set_verbose(true);
        assert!(client.is_verbose());

        assert!(
            OpenListClient::new("http://example.test")
                .with_verbose(true)
                .is_verbose()
        );
    }

    #[test]
    fn redacts_sensitive_json_keys_recursively() {
        let mut value = serde_json::json!({
            "username": "admin",
            "password": "hunter2",
            "nested": {
                "token": "token-value",
                "otp_code": "123456",
                "safe": "visible",
            },
            "items": [{"secret": "secret-value"}],
        });

        redact_json(&mut value);

        assert_eq!(value["username"], "admin");
        assert_eq!(value["password"], "<redacted>");
        assert_eq!(value["nested"]["token"], "<redacted>");
        assert_eq!(value["nested"]["otp_code"], "<redacted>");
        assert_eq!(value["nested"]["safe"], "visible");
        assert_eq!(value["items"][0]["secret"], "<redacted>");
    }

    #[test]
    fn identifies_sensitive_headers() {
        assert!(is_sensitive_header("authorization"));
        assert!(is_sensitive_header("cookie"));
        assert!(!is_sensitive_header("content-type"));
    }

    #[test]
    fn percent_encodes_only_unsafe_path_bytes() {
        assert_eq!(
            encode_path("/tianyi/packages/report-2026.pdf"),
            "/tianyi/packages/report-2026.pdf"
        );
        assert_eq!(
            encode_path("/tianyi/packages/desktop screen/报告 1.txt"),
            "/tianyi/packages/desktop%20screen/%E6%8A%A5%E5%91%8A%201.txt"
        );
        assert_eq!(encode_path("/a/50%.txt"), "/a/50%25.txt");
    }

    #[test]
    fn body_preview_flattens_and_truncates() {
        assert_eq!(body_preview(b""), "<empty>");
        assert_eq!(body_preview(b"error code: 504"), "error code: 504");
        assert_eq!(body_preview(b"a\n\n  b\tc"), "a b c");
        assert_eq!(body_preview(b"\0\0\0"), "<binary body>");
        assert!(body_preview(&[b'x'; 500]).ends_with('…'));
    }
}
