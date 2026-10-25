use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Generic API envelope ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T = serde_json::Value> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

// ── Pagination ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PageReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListData<T> {
    pub content: Option<Vec<T>>,
    pub total: Option<i64>,
}

// ── Auth ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginHashRequest {
    pub username: String,
    /// SHA256 hash of password
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verify2FARequest {
    pub code: String,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generate2FAData {
    pub secret: Option<String>,
    pub qr: Option<String>,
}

// ── User ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    /// 0=General, 1=Guest, 2=Admin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Permission bitmap (see API docs for bit definitions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCurrentUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

// ── SSH Keys ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshPublicKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddSshKeyRequest {
    pub title: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteSshKeyRequest {
    pub id: i64,
}

// ── File System ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dir: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
    /// 0=Unknown, 1=Folder, 2=Video, 3=Audio, 4=Text, 5=Image
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashinfo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_info: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_details: Option<StorageDetails>,
    /// Direct download URL returned by `fs/get` for files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_space: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_space: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsListData {
    pub content: Option<Vec<FsObject>>,
    pub total: Option<i64>,
    pub readme: Option<String>,
    pub header: Option<String>,
    pub write: Option<bool>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsGetRequest {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsMkdirRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsRenameRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenameObject {
    pub src_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsBatchRenameRequest {
    pub src_dir: String,
    pub rename_objects: Vec<RenameObject>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsRegexRenameRequest {
    pub src_dir: String,
    pub src_name_regex: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsMoveCopyRequest {
    pub src_dir: String,
    pub dst_dir: String,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsRemoveRequest {
    pub dir: String,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsRemoveEmptyDirRequest {
    pub src_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsSearchRequest {
    pub parent: String,
    pub keywords: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsDirsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_root: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsOtherRequest {
    pub path: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddOfflineDownloadRequest {
    pub path: String,
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsArchiveDecompressRequest {
    pub src_dir: String,
    pub name: String,
    pub dst_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsArchiveMetaRequest {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsArchiveListRequest {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
}

// ── Storage ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Storage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expiration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_index: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_sign: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webdav_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_proxy_url: Option<String>,
}

// ── Driver ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
}

// ── Settings ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveSettingsRequest(pub Vec<Setting>);

#[derive(Debug, Clone, Serialize)]
pub struct DeleteSettingRequest {
    pub key: String,
}

// ── Meta ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_sub: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w_sub: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h_sub: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_sub: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_sub: Option<bool>,
}

// ── Sharing ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sharing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Shared paths as returned by the server (`files` in the JSON payload).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    /// First shared path, kept for display purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Access password (`pwd` in the JSON payload).
    #[serde(rename = "pwd", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Expiry timestamp (`expires` in the JSON payload).
    #[serde(rename = "expires", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Normalise a share expiration value to the RFC 3339 timestamp expected by
/// the server.
///
/// `None`, an empty string and `"0"` mean "never expires". A duration such as
/// `30m`, `12h` or `7d` is resolved against the current time. Any other value
/// is passed through untouched so callers can supply their own timestamp.
pub fn parse_expiration(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "0" {
        return None;
    }

    let (amount, unit) = value.split_at(value.len().saturating_sub(1));
    if let Ok(amount) = amount.parse::<i64>()
        && amount > 0
    {
        let seconds = match unit {
            "s" => Some(amount),
            "m" => Some(amount * 60),
            "h" => Some(amount * 60 * 60),
            "d" => Some(amount * 24 * 60 * 60),
            _ => None,
        };
        if let Some(seconds) = seconds {
            let expires = chrono::Utc::now() + chrono::Duration::seconds(seconds);
            return Some(expires.to_rfc3339());
        }
    }

    Some(value.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSharingRequest {
    /// Shared paths (`files` in the JSON payload).
    #[serde(rename = "files")]
    pub paths: Vec<String>,
    #[serde(rename = "pwd", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Expiry as an RFC 3339 timestamp. Use [`parse_expiration`] to convert
    /// values such as `0` (never) or `7d`.
    #[serde(rename = "expires", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSharingRequest {
    pub id: String,
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSharingData {
    pub id: Option<String>,
    pub url: Option<String>,
}

// ── Index / Scan ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UpdateIndexRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexProgress {
    pub done: Option<bool>,
    pub total: Option<i64>,
    pub current: Option<i64>,
}

// ── ID-based requests ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct IdRequest {
    pub id: i64,
}
