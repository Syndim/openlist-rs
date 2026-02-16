use crate::client::OpenListClient;
use crate::error::Result;
use crate::models::*;

pub struct UserApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl UserApi<'_> {
    /// Retrieve information about the currently authenticated user.
    pub async fn me(&self) -> Result<ApiResponse<User>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/me"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Update password or other settings for current user.
    pub async fn update_me(
        &self,
        req: &UpdateCurrentUserRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/me/update"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Get list of SSH public keys for current user.
    pub async fn list_ssh_keys(&self) -> Result<ApiResponse<Vec<SshPublicKey>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/me/sshkey/list"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Add a new SSH public key for current user.
    pub async fn add_ssh_key(
        &self,
        req: &AddSshKeyRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/me/sshkey/add"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove an SSH public key from current user.
    pub async fn delete_ssh_key(
        &self,
        req: &DeleteSshKeyRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/me/sshkey/delete"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }
}
