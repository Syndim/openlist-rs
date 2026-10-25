use crate::client::OpenListClient;
use crate::error::Result;
use crate::models::*;

pub struct SharingApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl SharingApi<'_> {
    /// Get paginated list of file shares created by current user.
    pub async fn list(&self, req: &PageReq) -> Result<ApiResponse<ListData<Sharing>>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/list"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve specific share information.
    pub async fn get(&self, id: &str) -> Result<ApiResponse<Sharing>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/share/get"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Create a new shareable link for files/folders.
    pub async fn create(&self, req: &CreateSharingRequest) -> Result<ApiResponse<Sharing>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/create"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Modify existing share configuration.
    pub async fn update(&self, req: &UpdateSharingRequest) -> Result<ApiResponse<Sharing>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/update"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove a file share.
    pub async fn delete(&self, id: &str) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/delete"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Re-enable a disabled share.
    pub async fn enable(&self, id: &str) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/enable"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Temporarily disable a share.
    pub async fn disable(&self, id: &str) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/share/disable"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }
}
