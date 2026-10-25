use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenListError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error {code}: {message}")]
    Api { code: i32, message: String },

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// The endpoint answered with something other than an OpenList JSON body,
    /// which usually means an intermediary (proxy, CDN or gateway) answered
    /// instead of OpenList. The status and a body preview are kept because that
    /// is the only evidence identifying the hop that failed.
    #[error("{message}")]
    NonJsonResponse {
        status: u16,
        content_type: String,
        body: String,
        message: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported task kind: {0}")]
    InvalidTaskKind(String),
}

pub type Result<T> = std::result::Result<T, OpenListError>;

impl OpenListError {
    pub(crate) fn non_json_response(
        status: u16,
        content_type: String,
        body: String,
        source: serde_json::Error,
    ) -> Self {
        let rendered_type = if content_type.is_empty() {
            "no content type"
        } else {
            content_type.as_str()
        };
        let hint = match status {
            502 | 503 | 504 | 524 => {
                "; the request did not get an answer from OpenList (gateway or origin timeout)"
            }
            403 => "; a proxy or WAF answered instead of the OpenList API",
            _ => "",
        };
        let message = format!(
            "HTTP {status} ({rendered_type}), body was not the OpenList JSON API response: {body}{hint}"
        );
        Self::NonJsonResponse {
            status,
            content_type,
            body,
            message,
            source,
        }
    }
}
