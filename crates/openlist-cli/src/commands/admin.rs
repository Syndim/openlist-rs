use anyhow::Result;
use openlist::OpenListClient;
use openlist::models::*;

use crate::cli::{AdminCommand, StoragesCommand, UsersCommand};
use crate::output;

pub async fn run(client: &OpenListClient, command: AdminCommand, json: bool) -> Result<()> {
    match command {
        AdminCommand::Users(command) => users(client, command, json).await,
        AdminCommand::Storages(command) => storages(client, command, json).await,
    }
}

async fn users(client: &OpenListClient, command: UsersCommand, json: bool) -> Result<()> {
    match command {
        UsersCommand::List { page, per_page } => {
            let resp = client.admin().list_users(page, per_page).await?;
            if json {
                return output::emit_json(&resp);
            }

            let Some(data) = resp.data else {
                println!("{}", resp.message);
                return Ok(());
            };
            let users = data.content.unwrap_or_default();
            if users.is_empty() {
                println!("(no users)");
                return Ok(());
            }

            let rows = users
                .into_iter()
                .map(|user| {
                    vec![
                        user.id.map(|id| id.to_string()).unwrap_or_default(),
                        user.username.unwrap_or_default(),
                        user.role.map(|role| role.to_string()).unwrap_or_default(),
                        user.permission
                            .map(|permission| permission.to_string())
                            .unwrap_or_default(),
                        user.disabled
                            .map(|disabled| disabled.to_string())
                            .unwrap_or_default(),
                        user.base_path.unwrap_or_default(),
                    ]
                })
                .collect();

            output::table(
                &[
                    "ID",
                    "USERNAME",
                    "ROLE",
                    "PERMISSION",
                    "DISABLED",
                    "BASE_PATH",
                ],
                rows,
            );
            Ok(())
        }
        UsersCommand::Get { id } => {
            let resp = client.admin().get_user(id).await?;
            if json {
                return output::emit_json(&resp);
            }

            let Some(user) = resp.data else {
                println!("{}", resp.message);
                return Ok(());
            };
            let rows = vec![
                output::field("id", &user.id),
                output::field("username", &user.username),
                output::field("base_path", &user.base_path),
                output::field("role", &user.role),
                output::field("permission", &user.permission),
                output::field("disabled", &user.disabled),
                output::field("sso_id", &user.sso_id),
                output::field("otp", &user.otp),
            ];
            output::table(&["FIELD", "VALUE"], rows);
            Ok(())
        }
        UsersCommand::Create {
            username,
            password,
            base_path,
            role,
            permission,
            disabled,
        } => {
            let resp = client
                .admin()
                .create_user(&CreateUserRequest {
                    username,
                    password,
                    base_path,
                    role,
                    permission,
                    disabled: if disabled { Some(true) } else { None },
                })
                .await?;
            emit(resp, json)
        }
        UsersCommand::Delete { id } => {
            let resp = client.admin().delete_user(id).await?;
            emit(resp, json)
        }
    }
}

async fn storages(client: &OpenListClient, command: StoragesCommand, json: bool) -> Result<()> {
    match command {
        StoragesCommand::List { page, per_page } => {
            let resp = client.admin().list_storages(page, per_page).await?;
            if json {
                return output::emit_json(&resp);
            }

            let Some(data) = resp.data else {
                println!("{}", resp.message);
                return Ok(());
            };
            let storages = data.content.unwrap_or_default();
            if storages.is_empty() {
                println!("(no storage mounts)");
                return Ok(());
            }

            let rows = storages
                .into_iter()
                .map(|storage| {
                    vec![
                        storage.id.map(|id| id.to_string()).unwrap_or_default(),
                        storage.mount_path.unwrap_or_default(),
                        storage.driver.unwrap_or_default(),
                        storage.status.unwrap_or_default(),
                        storage
                            .disabled
                            .map(|disabled| disabled.to_string())
                            .unwrap_or_default(),
                        storage.remark.unwrap_or_default(),
                    ]
                })
                .collect();

            output::table(
                &["ID", "MOUNT_PATH", "DRIVER", "STATUS", "DISABLED", "REMARK"],
                rows,
            );
            Ok(())
        }
        StoragesCommand::Get { id } => {
            let resp = client.admin().get_storage(id).await?;
            if json {
                return output::emit_json(&resp);
            }

            let Some(storage) = resp.data else {
                println!("{}", resp.message);
                return Ok(());
            };
            let rows = vec![
                output::field("id", &storage.id),
                output::field("mount_path", &storage.mount_path),
                output::field("driver", &storage.driver),
                output::field("status", &storage.status),
                output::field("disabled", &storage.disabled),
                output::field("order", &storage.order),
                output::field("remark", &storage.remark),
                output::field("cache_expiration", &storage.cache_expiration),
                output::field("webdav_policy", &storage.webdav_policy),
            ];
            output::table(&["FIELD", "VALUE"], rows);
            Ok(())
        }
        StoragesCommand::Enable { id } => {
            let resp = client.admin().enable_storage(id).await?;
            emit(resp, json)
        }
        StoragesCommand::Disable { id } => {
            let resp = client.admin().disable_storage(id).await?;
            emit(resp, json)
        }
        StoragesCommand::StorageCreate {
            root,
            mount_path,
            cache_expiration,
        } => {
            let root = std::path::absolute(&root).unwrap_or(root);
            let storage = Storage {
                mount_path: Some(mount_path),
                order: Some(0),
                driver: Some("Local".into()),
                cache_expiration: cache_expiration.or(Some(30)),
                status: Some(String::new()),
                addition: Some(
                    serde_json::json!({
                        "root_folder_path": root,
                        "thumbnail": false,
                    })
                    .to_string(),
                ),
                remark: Some("created by openlist-cli".into()),
                ..Storage::default()
            };
            let resp = client.admin().create_storage(&storage).await?;
            emit(resp, json)
        }
    }
}

fn emit(resp: ApiResponse<serde_json::Value>, json: bool) -> Result<()> {
    if json {
        output::emit_json(&resp)
    } else {
        output::emit_message(&resp)
    }
}
