use bytes::Bytes;

use crate::client::OpenListClient;
use crate::error::Result;
use crate::models::*;

pub struct FsApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl FsApi<'_> {
    /// Get list of files and directories at specified path with pagination.
    pub async fn list(&self, req: &FsListRequest) -> Result<ApiResponse<FsListData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/list"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve metadata for a specific file or directory.
    pub async fn get(&self, req: &FsGetRequest) -> Result<ApiResponse<FsObject>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/get"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Search for files/folders by name (requires search index to be built).
    pub async fn search(&self, req: &FsSearchRequest) -> Result<ApiResponse<ListData<FsObject>>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/search"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve directory structure (folders only) for navigation.
    pub async fn dirs(&self, req: &FsDirsRequest) -> Result<ApiResponse<Vec<DirEntry>>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/dirs"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve provider-specific operations available for a file/folder.
    pub async fn other(&self, req: &FsOtherRequest) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/other"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Create a new directory at specified path.
    pub async fn mkdir(&self, req: &FsMkdirRequest) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/mkdir"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Rename a file or directory.
    pub async fn rename(&self, req: &FsRenameRequest) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/rename"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Rename multiple files using a pattern.
    pub async fn batch_rename(
        &self,
        req: &FsBatchRenameRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/batch_rename"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Rename files using regular expression pattern matching.
    pub async fn regex_rename(
        &self,
        req: &FsRegexRenameRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/regex_rename"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Move one or more files/folders to another location.
    pub async fn move_files(
        &self,
        req: &FsMoveCopyRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/move"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Move files/folders recursively (preserves directory structure).
    pub async fn recursive_move(
        &self,
        req: &FsMoveCopyRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/recursive_move"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Copy one or more files/folders to another location.
    pub async fn copy(&self, req: &FsMoveCopyRequest) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/copy"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Delete one or more files or folders.
    pub async fn remove(&self, req: &FsRemoveRequest) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/remove"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Recursively remove empty directories.
    pub async fn remove_empty_directory(
        &self,
        req: &FsRemoveEmptyDirRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/remove_empty_directory"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Upload file using streaming. `path` is the destination file path.
    /// `as_task` uploads as background task. `last_modified` is Unix timestamp in ms.
    pub async fn put(
        &self,
        path: &str,
        body: Bytes,
        as_task: Option<bool>,
        last_modified: Option<&str>,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let mut req = self
            .client
            .http
            .put(self.client.url("/api/fs/put"))
            .headers(self.client.auth_headers())
            .header("path", path)
            .body(body);
        if let Some(t) = as_task {
            req = req.header("As-Task", t.to_string());
        }
        if let Some(lm) = last_modified {
            req = req.header("Last-Modified", lm);
        }
        let resp = req.send().await?;
        self.client.handle_response(resp).await
    }

    /// Upload file using multipart form data.
    pub async fn form_upload(
        &self,
        path: &str,
        file_name: &str,
        body: Bytes,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let part = reqwest::multipart::Part::bytes(body.to_vec()).file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);
        let resp = self
            .client
            .http
            .put(self.client.url("/api/fs/form"))
            .headers(self.client.auth_headers())
            .header("path", path)
            .multipart(form)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Create an offline download task (HTTP/magnet/torrent).
    pub async fn add_offline_download(
        &self,
        req: &AddOfflineDownloadRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/add_offline_download"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Extract archive file (zip/rar/7z/tar/tar.gz etc.).
    pub async fn archive_decompress(
        &self,
        req: &FsArchiveDecompressRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/archive/decompress"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// Retrieve metadata of archive file without extracting.
    pub async fn archive_meta(
        &self,
        req: &FsArchiveMetaRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/archive/meta"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }

    /// List files inside an archive without extracting.
    pub async fn archive_list(
        &self,
        req: &FsArchiveListRequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/fs/archive/list"))
            .headers(self.client.auth_headers())
            .json(req)
            .send()
            .await?;
        self.client.handle_response(resp).await
    }
}
