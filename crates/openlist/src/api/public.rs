use std::collections::HashMap;

use crate::client::OpenListClient;
use crate::error::Result;
use crate::models::*;

pub struct PublicApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl PublicApi<'_> {
    /// Retrieve public system settings (no authentication required).
    pub async fn settings(&self) -> Result<ApiResponse<HashMap<String, String>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/public/settings"))
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// List configured offline download tools (aria2, qbittorrent, etc.).
    pub async fn offline_download_tools(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/public/offline_download_tools"))
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// List archive file extensions that can be extracted.
    pub async fn archive_extensions(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/public/archive_extensions"))
            .send()
            .await?;
        self.client.handle_response(resp).await
    }
}
