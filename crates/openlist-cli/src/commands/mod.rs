use anyhow::Result;
use openlist::OpenListClient;

use crate::cli::{Cli, Command, DEFAULT_SERVER};
use crate::config::Config;

mod admin;
mod auth;
mod fs;
mod share;

pub async fn run(cli: Cli) -> Result<()> {
    let Cli {
        server,
        token,
        json,
        command,
    } = cli;

    let mut config = Config::load()?;
    let server = server
        .or_else(|| config.server.clone())
        .unwrap_or_else(|| DEFAULT_SERVER.to_string());

    let mut client = OpenListClient::new(server.clone());
    let token = token.or_else(|| config.token.clone());
    let has_token = token.is_some();
    if let Some(token) = token {
        client.set_token(token);
    }

    match command {
        Command::Login {
            username,
            password,
            otp_code,
        } => {
            auth::login(
                &client,
                &mut config,
                &server,
                username,
                password,
                otp_code,
                json,
            )
            .await
        }
        Command::Logout => auth::logout(&client, &mut config, has_token, json).await,
        Command::Me => auth::me(&client, json).await,

        Command::Ls {
            path,
            refresh,
            page,
            per_page,
        } => fs::ls(&client, path, refresh, page, per_page, json).await,
        Command::Stat { path } => fs::stat(&client, path, json).await,
        Command::Mkdir { path } => fs::mkdir(&client, path, json).await,
        Command::Rename { path, new_name } => fs::rename(&client, path, new_name, json).await,
        Command::Mv {
            src_dir,
            dst_dir,
            names,
        } => fs::move_entries(&client, src_dir, dst_dir, names, json).await,
        Command::Cp {
            src_dir,
            dst_dir,
            names,
        } => fs::copy(&client, src_dir, dst_dir, names, json).await,
        Command::Rm { dir, names, yes } => fs::remove(&client, dir, names, yes, json).await,
        Command::Search {
            parent,
            keywords,
            scope,
            page,
            per_page,
        } => fs::search(&client, parent, keywords, scope, page, per_page, json).await,
        Command::Upload {
            local,
            remote_dir,
            name,
        } => fs::upload(&client, local, remote_dir, name, json).await,

        Command::Share(command) => share::run(&client, command, json).await,
        Command::Admin(command) => admin::run(&client, command, json).await,
    }
}
