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
        } => create(client, paths, password, expiration, json).await,
        ShareCommand::Update {
            id,
            path,
            password,
            expiration,
            disable,
        } => update(client, id, path, password, expiration, disable, json).await,
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
                share.path.unwrap_or_default(),
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

async fn update(
    client: &OpenListClient,
    id: String,
    path: Option<String>,
    password: Option<String>,
    expiration: Option<String>,
    disabled: Option<bool>,
    json: bool,
) -> Result<()> {
    let share = Sharing {
        id: Some(id),
        path,
        url: None,
        password,
        expiration,
        created_at: None,
        disabled,
    };
    let resp = client.sharing().update(&share).await?;
    emit(resp, json)
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
