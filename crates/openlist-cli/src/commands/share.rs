use anyhow::Result;
use openlist::OpenListClient;
use openlist::models::*;

use crate::cli::ShareCommand;
use crate::output;

pub async fn run(client: &OpenListClient, command: ShareCommand, json: bool) -> Result<()> {
    match command {
        ShareCommand::List { page, per_page } => list(client, page, per_page, json).await,
        ShareCommand::Get { id } => get(client, id, json).await,
        ShareCommand::Create {
            paths,
            password,
            expiration,
        } => {
            let expiration = expiration.as_deref().and_then(parse_expiration);
            create(client, paths, password, expiration, json).await
        }
        ShareCommand::Update {
            id,
            path,
            password,
            expiration,
            disable,
        } => {
            // "0"/empty mean "never expires". The server rejects an empty
            // timestamp, so the field is omitted in that case.
            let expiration = match expiration.as_deref() {
                None => Expiration::Keep,
                Some(value) => match parse_expiration(value) {
                    Some(expiration) => Expiration::Set(expiration),
                    None => Expiration::Clear,
                },
            };
            update(client, id, path, password, expiration, disable, json).await
        }
        ShareCommand::Delete { id } => delete(client, id, json).await,
        ShareCommand::Enable { id } => enable(client, id, json).await,
        ShareCommand::Disable { id } => disable(client, id, json).await,
    }
}

async fn list(
    client: &OpenListClient,
    page: Option<i64>,
    per_page: Option<i64>,
    json: bool,
) -> Result<()> {
    let resp = client.sharing().list(&PageReq { page, per_page }).await?;
    if json {
        return output::emit_json(&resp);
    }

    let Some(data) = resp.data else {
        println!("{}", resp.message);
        return Ok(());
    };
    let shares = data.content.unwrap_or_default();
    if shares.is_empty() {
        println!("(no share links)");
        return Ok(());
    }

    let rows = shares
        .into_iter()
        .map(|share| {
            vec![
                share.id.unwrap_or_default(),
                share
                    .files
                    .as_ref()
                    .map(|f| f.join(", "))
                    .unwrap_or_default(),
                share.url.unwrap_or_default(),
                if share.disabled.unwrap_or(false) {
                    "disabled".to_string()
                } else {
                    "enabled".to_string()
                },
                share.expiration.unwrap_or_default(),
                share.created_at.unwrap_or_default(),
            ]
        })
        .collect();

    output::table(
        &["ID", "PATH", "URL", "STATUS", "EXPIRATION", "CREATED"],
        rows,
    );
    Ok(())
}

async fn get(client: &OpenListClient, id: String, json: bool) -> Result<()> {
    let resp = client.sharing().get(&id).await?;
    if json {
        return output::emit_json(&resp);
    }

    let Some(share) = resp.data else {
        println!("{}", resp.message);
        return Ok(());
    };

    let rows = vec![
        output::field("id", &share.id),
        output::field("path", &share.path),
        output::field("url", &share.url),
        output::field("password", &share.password),
        output::field("expiration", &share.expiration),
        output::field("created_at", &share.created_at),
        output::field("disabled", &share.disabled),
    ];
    output::table(&["FIELD", "VALUE"], rows);
    Ok(())
}

async fn create(
    client: &OpenListClient,
    paths: Vec<String>,
    password: Option<String>,
    expiration: Option<String>,
    json: bool,
) -> Result<()> {
    let resp = client
        .sharing()
        .create(&CreateSharingRequest {
            paths,
            password,
            expiration,
        })
        .await?;
    if json {
        return output::emit_json(&resp);
    }

    match resp.data {
        Some(data) => {
            println!("{}", resp.message);
            if let Some(id) = data.id {
                println!("id:  {id}");
            }
            if let Some(url) = data.url {
                println!("url: {url}");
            }
        }
        None => println!("{}", resp.message),
    }
    Ok(())
}

/// Expiration changes requested through `--expiration`.
enum Expiration {
    /// Leave the expiry untouched.
    Keep,
    /// Set an explicit RFC 3339 timestamp.
    Set(String),
    /// `--expiration 0` / empty: the share never expires.
    Clear,
}

async fn update(
    client: &OpenListClient,
    id: String,
    path: Option<String>,
    password: Option<String>,
    expiration: Expiration,
    disabled: Option<bool>,
    json: bool,
) -> Result<()> {
    // The server replaces the whole share on update, so start from the current
    // state and apply the requested changes.
    let current = client.sharing().get(&id).await?.data.unwrap_or_default();
    let files = match path {
        Some(path) => vec![path],
        None => current.files.unwrap_or_default(),
    };
    let resp = client
        .sharing()
        .update(&UpdateSharingRequest {
            id,
            files,
            pwd: password.or(current.password),
            expires: match expiration {
                Expiration::Set(expiration) => Some(expiration),
                Expiration::Clear => None,
                Expiration::Keep => current.expiration,
            },
            disabled: disabled.or(current.disabled),
        })
        .await?;
    if json {
        return output::emit_json(&resp);
    }
    println!("{}", resp.message);
    if let Some(share) = resp.data {
        if let Some(id) = share.id {
            println!("id:  {id}");
        }
        if let Some(password) = share.password {
            println!("password: {password}");
        }
        if let Some(expiration) = share.expiration {
            println!("expiration: {expiration}");
        }
        println!(
            "disabled: {}",
            share.disabled.map(|d| d.to_string()).unwrap_or_default()
        );
    }
    Ok(())
}

async fn delete(client: &OpenListClient, id: String, json: bool) -> Result<()> {
    let resp = client.sharing().delete(&id).await?;
    emit(resp, json)
}

async fn enable(client: &OpenListClient, id: String, json: bool) -> Result<()> {
    let resp = client.sharing().enable(&id).await?;
    emit(resp, json)
}

async fn disable(client: &OpenListClient, id: String, json: bool) -> Result<()> {
    let resp = client.sharing().disable(&id).await?;
    emit(resp, json)
}

fn emit(resp: ApiResponse<serde_json::Value>, json: bool) -> Result<()> {
    if json {
        output::emit_json(&resp)
    } else {
        output::emit_message(&resp)
    }
}
