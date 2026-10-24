use anyhow::{Result, anyhow};
use openlist::OpenListClient;
use openlist::models::LoginRequest;

use crate::config::Config;
use crate::output;

pub async fn login(
    client: &OpenListClient,
    config: &mut Config,
    server: &str,
    username: String,
    password: String,
    otp_code: Option<String>,
    json: bool,
) -> Result<()> {
    let resp = client
        .auth()
        .login(&LoginRequest {
            username,
            password,
            otp_code,
        })
        .await?;

    let token = resp
        .data
        .as_ref()
        .map(|data| data.token.clone())
        .filter(|token| !token.is_empty())
        .ok_or_else(|| anyhow!("login succeeded but the server returned no token"))?;

    config.set_credentials(server, token);
    config.save()?;

    if json {
        output::emit_json(&resp)
    } else {
        println!(
            "Logged in. Credentials stored in {}",
            Config::path()?.display()
        );
        Ok(())
    }
}

pub async fn logout(
    client: &OpenListClient,
    config: &mut Config,
    has_token: bool,
    json: bool,
) -> Result<()> {
    if has_token && let Err(err) = client.auth().logout().await {
        eprintln!("warning: server-side logout failed: {err}");
    }

    config.clear_token();
    config.save()?;

    if json {
        output::emit_json(&serde_json::json!({ "message": "logged out" }))
    } else {
        println!("Logged out.");
        Ok(())
    }
}

pub async fn me(client: &OpenListClient, json: bool) -> Result<()> {
    let resp = client.user().me().await?;
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
