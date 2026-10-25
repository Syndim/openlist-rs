use serde_json::Value;

use crate::client::{OpenListClient, RequestBuilderExt};
use crate::error::{OpenListError, Result};
use crate::models::{ApiResponse, TaskInfo};

/// Task groups exposed by OpenList. The value is used as the URL segment.
pub const TASK_KINDS: [&str; 7] = [
    "upload",
    "copy",
    "move",
    "offline_download",
    "offline_download_transfer",
    "decompress",
    "decompress_upload",
];

pub struct TaskApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl TaskApi<'_> {
    fn kind(kind: &str) -> Result<&str> {
        if TASK_KINDS.contains(&kind) {
            Ok(kind)
        } else {
            Err(OpenListError::InvalidTaskKind(kind.to_string()))
        }
    }

    fn url(kind: &str, action: &str) -> Result<String> {
        Ok(format!("/api/task/{}/{action}", Self::kind(kind)?))
    }

    /// Tasks that are still pending, running or waiting for a retry.
    pub async fn undone(&self, kind: &str) -> Result<ApiResponse<Vec<TaskInfo>>> {
        let resp = self
            .client
            .http
            .get(self.client.url(&Self::url(kind, "undone")?))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Tasks that succeeded, failed or were canceled.
    pub async fn done(&self, kind: &str) -> Result<ApiResponse<Vec<TaskInfo>>> {
        let resp = self
            .client
            .http
            .get(self.client.url(&Self::url(kind, "done")?))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Look up one task by the id returned when it was queued.
    pub async fn info(&self, kind: &str, id: &str) -> Result<ApiResponse<TaskInfo>> {
        let resp = self
            .client
            .http
            .post(self.client.url(&Self::url(kind, "info")?))
            .headers(self.client.auth_headers())
            .query(&[("tid", id)])
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    pub async fn cancel(&self, kind: &str, id: &str) -> Result<ApiResponse<Value>> {
        self.action(kind, "cancel", id).await
    }

    pub async fn retry(&self, kind: &str, id: &str) -> Result<ApiResponse<Value>> {
        self.action(kind, "retry", id).await
    }

    async fn action(&self, kind: &str, action: &str, id: &str) -> Result<ApiResponse<Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url(&Self::url(kind, action)?))
            .headers(self.client.auth_headers())
            .query(&[("tid", id)])
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }
}
