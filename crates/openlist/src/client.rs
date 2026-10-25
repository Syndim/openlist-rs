use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::error::{OpenListError, Result};
use crate::models::ApiResponse;

#[derive(Clone)]
pub struct OpenListClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) token: Option<String>,
}

impl OpenListClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub fn clear_token(&mut self) {
        self.token = None;
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
        let api_resp: ApiResponse<T> = resp.json().await?;
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
}
