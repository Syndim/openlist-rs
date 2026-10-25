//! # OpenList SDK
//!
//! Rust SDK for the [OpenList](https://github.com/OpenListTeam/OpenList) API.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use openlist::{OpenListClient, models::LoginRequest};
//!
//! #[tokio::main]
//! async fn main() -> openlist::Result<()> {
//!     let mut client = OpenListClient::new("http://localhost:5244");
//!
//!     // Login
//!     let resp = client.auth().login(&LoginRequest {
//!         username: "admin".into(),
//!         password: "password".into(),
//!         otp_code: None,
//!     }).await?;
//!     client.set_token(resp.data.unwrap().token);
//!
//!     // List files
//!     let files = client.fs().list(&openlist::models::FsListRequest {
//!         path: Some("/".into()),
//!         password: None,
//!         refresh: None,
//!         page: None,
//!         per_page: None,
//!     }).await?;
//!     println!("{:?}", files.data);
//!
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod error;
pub mod models;

pub use client::OpenListClient;
pub use error::{OpenListError, Result};
