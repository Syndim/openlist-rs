use crate::client::OpenListClient;
use crate::error::Result;
use crate::models::*;

pub struct AdminApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl AdminApi<'_> {
    // ── Users ────────────────────────────────────────────────────────

    /// Get paginated list of all users.
    pub async fn list_users(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<ApiResponse<ListData<User>>> {
        let mut req = self
            .client
            .http
            .get(self.client.url("/api/admin/user/list"))
            .headers(self.client.auth_headers());
        if let Some(p) = page {
            req = req.query(&[("page", p)]);
        }
        if let Some(pp) = per_page {
            req = req.query(&[("per_page", pp)]);
        }
        let resp = req.send().await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve specific user information.
    pub async fn get_user(&self, id: i64) -> Result<ApiResponse<User>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/user/get"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Create a new user account.
    pub async fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/create"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Update user information.
    pub async fn update_user(&self, user: &User) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/update"))
            .headers(self.client.auth_headers())
            .json(user)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Delete a user account.
    pub async fn delete_user(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/delete"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Disable 2FA for a specific user.
    pub async fn cancel_2fa(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/cancel_2fa"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Clear cached data for a specific user.
    pub async fn del_user_cache(&self, username: &str) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/del_cache"))
            .headers(self.client.auth_headers())
            .query(&[("username", username)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Get SSH keys for any user (admin).
    pub async fn list_user_ssh_keys(&self, user_id: i64) -> Result<ApiResponse<Vec<SshPublicKey>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/user/sshkey/list"))
            .headers(self.client.auth_headers())
            .query(&[("user_id", user_id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Delete SSH key for any user (admin).
    pub async fn delete_user_ssh_key(
        &self,
        req: &DeleteSshKeyRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/user/sshkey/delete"))
            .headers(self.client.auth_headers())
            .query(&[("id", req.id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Storages ─────────────────────────────────────────────────────

    /// Get list of all configured storage mounts.
    pub async fn list_storages(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<ApiResponse<ListData<Storage>>> {
        let mut req = self
            .client
            .http
            .get(self.client.url("/api/admin/storage/list"))
            .headers(self.client.auth_headers());
        if let Some(p) = page {
            req = req.query(&[("page", p)]);
        }
        if let Some(pp) = per_page {
            req = req.query(&[("per_page", pp)]);
        }
        let resp = req.send().await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve specific storage configuration.
    pub async fn get_storage(&self, id: i64) -> Result<ApiResponse<Storage>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/storage/get"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Add a new storage mount.
    pub async fn create_storage(
        &self,
        storage: &Storage,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/create"))
            .headers(self.client.auth_headers())
            .json(storage)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Modify existing storage configuration.
    pub async fn update_storage(
        &self,
        storage: &Storage,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/update"))
            .headers(self.client.auth_headers())
            .json(storage)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove a storage mount.
    pub async fn delete_storage(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/delete"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Enable a disabled storage mount.
    pub async fn enable_storage(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/enable"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Temporarily disable a storage mount.
    pub async fn disable_storage(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/disable"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Force reload all storage mounts.
    pub async fn load_all_storages(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/storage/load_all"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Drivers ──────────────────────────────────────────────────────

    /// Get list of available storage drivers with their configurations.
    pub async fn list_drivers(&self) -> Result<ApiResponse<Vec<DriverInfo>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/driver/list"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Get list of available driver names.
    pub async fn list_driver_names(&self) -> Result<ApiResponse<Vec<String>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/driver/names"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Get configuration schema for a specific driver.
    pub async fn get_driver_info(&self, driver: &str) -> Result<ApiResponse<DriverInfo>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/driver/info"))
            .headers(self.client.auth_headers())
            .query(&[("driver", driver)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Settings ─────────────────────────────────────────────────────

    /// Get all system settings.
    pub async fn list_settings(&self) -> Result<ApiResponse<Vec<Setting>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/setting/list"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve a specific setting value.
    pub async fn get_setting(&self, key: &str) -> Result<ApiResponse<Setting>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/setting/get"))
            .headers(self.client.auth_headers())
            .query(&[("key", key)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Update one or more system settings.
    pub async fn save_settings(
        &self,
        settings: &[Setting],
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/setting/save"))
            .headers(self.client.auth_headers())
            .json(settings)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove a custom setting.
    pub async fn delete_setting(&self, key: &str) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/setting/delete"))
            .headers(self.client.auth_headers())
            .json(&DeleteSettingRequest {
                key: key.to_string(),
            })
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Generate a new API token.
    pub async fn reset_token(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/setting/reset_token"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Restore default settings.
    pub async fn default_settings(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/setting/default"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Metas ────────────────────────────────────────────────────────

    /// Get list of metadata configurations.
    pub async fn list_metas(&self) -> Result<ApiResponse<Vec<Meta>>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/meta/list"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve specific metadata configuration.
    pub async fn get_meta(&self, id: i64) -> Result<ApiResponse<Meta>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/meta/get"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Add new metadata configuration.
    pub async fn create_meta(&self, meta: &Meta) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/meta/create"))
            .headers(self.client.auth_headers())
            .json(meta)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Modify metadata configuration.
    pub async fn update_meta(&self, meta: &Meta) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/meta/update"))
            .headers(self.client.auth_headers())
            .json(meta)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove metadata configuration.
    pub async fn delete_meta(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/meta/delete"))
            .headers(self.client.auth_headers())
            .query(&[("id", id)])
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Search Index ─────────────────────────────────────────────────

    /// Build full-text search index for all storages.
    pub async fn build_index(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/index/build"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Update search index for specific paths.
    pub async fn update_index(
        &self,
        req: &UpdateIndexRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/index/update"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Stop current indexing operation.
    pub async fn stop_index(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/index/stop"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Delete all search index data.
    pub async fn clear_index(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/index/clear"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Check current indexing operation progress.
    pub async fn get_index_progress(&self) -> Result<ApiResponse<IndexProgress>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/index/progress"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    // ── Scan ─────────────────────────────────────────────────────────

    /// Start a manual scan.
    pub async fn start_scan(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/scan/start"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Stop a manual scan.
    pub async fn stop_scan(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/admin/scan/stop"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Get manual scan progress.
    pub async fn get_scan_progress(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/admin/scan/progress"))
            .headers(self.client.auth_headers())
            .send()
            .await?;
        self.client.handle_response(resp).await
    }
}
