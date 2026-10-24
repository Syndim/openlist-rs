use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use openlist::OpenListClient;
use openlist::models::*;

use crate::output;

pub async fn ls(
    client: &OpenListClient,
    path: Option<String>,
    refresh: bool,
    page: Option<i64>,
    per_page: Option<i64>,
    json: bool,
) -> Result<()> {
    let req = FsListRequest {
        path: Some(path.unwrap_or_else(|| "/".to_string())),
        password: None,
        refresh: if refresh { Some(true) } else { None },
        page,
        per_page,
    };
    let resp = client.fs().list(&req).await?;
    if json {
        return output::emit_json(&resp);
    }

    let Some(data) = resp.data else {
        println!("{}", resp.message);
        return Ok(());
    };
    let content = data.content.unwrap_or_default();
    if content.is_empty() {
        println!("(empty)");
        return Ok(());
    }

    let rows = content
        .into_iter()
        .map(|entry| {
            vec![
                if entry.is_dir.unwrap_or(false) {
                    "d".to_string()
                } else {
                    "-".to_string()
                },
                human_size(entry.size),
                entry.modified.unwrap_or_default(),
                entry.name.unwrap_or_default(),
            ]
        })
        .collect();

    output::table(&["TYPE", "SIZE", "MODIFIED", "NAME"], rows);

    if let Some(total) = data.total {
        println!(
            "\n{} {}",
            total,
            if total == 1 { "entry" } else { "entries" }
        );
    }
    Ok(())
}

pub async fn stat(client: &OpenListClient, path: String, json: bool) -> Result<()> {
    let resp = client
        .fs()
        .get(&FsGetRequest {
            path,
            password: None,
        })
        .await?;
    if json {
        return output::emit_json(&resp);
    }

    let Some(obj) = resp.data else {
        println!("{}", resp.message);
        return Ok(());
    };

    let rows = vec![
        output::field("path", &obj.path),
        output::field("name", &obj.name),
        output::field("size", &obj.size.map(|size| human_size(Some(size)))),
        output::field("is_dir", &obj.is_dir),
        output::field("modified", &obj.modified),
        output::field("created", &obj.created),
        output::field("file_type", &obj.file_type),
        output::field("sign", &obj.sign),
        output::field("thumb", &obj.thumb),
        output::field("hashinfo", &obj.hashinfo),
    ];
    output::table(&["FIELD", "VALUE"], rows);
    Ok(())
}

pub async fn mkdir(client: &OpenListClient, path: String, json: bool) -> Result<()> {
    let resp = client.fs().mkdir(&FsMkdirRequest { path }).await?;
    emit(resp, json)
}

pub async fn rename(
    client: &OpenListClient,
    path: String,
    new_name: String,
    json: bool,
) -> Result<()> {
    let resp = client
        .fs()
        .rename(&FsRenameRequest {
            path,
            name: new_name,
        })
        .await?;
    emit(resp, json)
}

pub async fn move_entries(
    client: &OpenListClient,
    src_dir: String,
    dst_dir: String,
    names: Vec<String>,
    json: bool,
) -> Result<()> {
    let resp = client
        .fs()
        .move_files(&FsMoveCopyRequest {
            src_dir,
            dst_dir,
            names,
        })
        .await?;
    emit(resp, json)
}

pub async fn copy(
    client: &OpenListClient,
    src_dir: String,
    dst_dir: String,
    names: Vec<String>,
    json: bool,
) -> Result<()> {
    let resp = client
        .fs()
        .copy(&FsMoveCopyRequest {
            src_dir,
            dst_dir,
            names,
        })
        .await?;
    emit(resp, json)
}

pub async fn remove(
    client: &OpenListClient,
    dir: String,
    names: Vec<String>,
    yes: bool,
    json: bool,
) -> Result<()> {
    if !yes && !confirm(&dir, &names)? {
        println!("Aborted.");
        return Ok(());
    }

    let resp = client.fs().remove(&FsRemoveRequest { dir, names }).await?;
    emit(resp, json)
}

pub async fn search(
    client: &OpenListClient,
    parent: String,
    keywords: String,
    scope: Option<i32>,
    page: Option<i64>,
    per_page: Option<i64>,
    json: bool,
) -> Result<()> {
    let resp = client
        .fs()
        .search(&FsSearchRequest {
            parent,
            keywords,
            scope,
            page,
            per_page,
        })
        .await?;
    if json {
        return output::emit_json(&resp);
    }

    let Some(data) = resp.data else {
        println!("{}", resp.message);
        return Ok(());
    };
    let content = data.content.unwrap_or_default();
    if content.is_empty() {
        println!("(no matches)");
        return Ok(());
    }

    let rows = content
        .into_iter()
        .map(|entry| {
            vec![
                if entry.is_dir.unwrap_or(false) {
                    "d".to_string()
                } else {
                    "-".to_string()
                },
                human_size(entry.size),
                entry.modified.unwrap_or_default(),
                entry.path.or(entry.name).unwrap_or_default(),
            ]
        })
        .collect();

    output::table(&["TYPE", "SIZE", "MODIFIED", "PATH"], rows);
    if let Some(total) = data.total {
        println!(
            "\n{} {}",
            total,
            if total == 1 { "match" } else { "matches" }
        );
    }
    Ok(())
}

pub async fn upload(
    client: &OpenListClient,
    local: PathBuf,
    remote_dir: String,
    name: Option<String>,
    json: bool,
) -> Result<()> {
    let file_name = match name {
        Some(name) => name,
        None => local
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .with_context(|| format!("cannot determine a file name from {}", local.display()))?,
    };

    let dest = join_remote(&remote_dir, &file_name);
    let body = tokio::fs::read(&local)
        .await
        .with_context(|| format!("failed to read {}", local.display()))?;

    let resp = client.fs().put(&dest, body.into(), None, None).await?;
    if json {
        return output::emit_json(&resp);
    }

    output::emit_message(&resp)?;
    println!("Uploaded {} -> {}", local.display(), dest);
    Ok(())
}

fn emit(resp: openlist::models::ApiResponse<serde_json::Value>, json: bool) -> Result<()> {
    if json {
        output::emit_json(&resp)
    } else {
        output::emit_message(&resp)
    }
}

fn confirm(dir: &str, names: &[String]) -> Result<bool> {
    print!(
        "Remove {} entr{} from {}? [y/N] ",
        names.len(),
        if names.len() == 1 { "y" } else { "ies" },
        dir
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn join_remote(dir: &str, name: &str) -> String {
    if dir.ends_with('/') {
        format!("{dir}{name}")
    } else {
        format!("{dir}/{name}")
    }
}

fn human_size(size: Option<i64>) -> String {
    let Some(bytes) = size else {
        return String::new();
    };
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    const UNITS: [&str; 5] = ["KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_sizes() {
        assert_eq!(human_size(None), "");
        assert_eq!(human_size(Some(0)), "0 B");
        assert_eq!(human_size(Some(512)), "512 B");
        assert_eq!(human_size(Some(1024)), "1.0 KiB");
        assert_eq!(human_size(Some(1536)), "1.5 KiB");
        assert_eq!(human_size(Some(1024 * 1024)), "1.0 MiB");
        assert_eq!(human_size(Some(3 * 1024 * 1024 * 1024)), "3.0 GiB");
    }

    #[test]
    fn joins_remote_paths() {
        assert_eq!(join_remote("/", "a.txt"), "/a.txt");
        assert_eq!(join_remote("/docs", "a.txt"), "/docs/a.txt");
        assert_eq!(join_remote("/docs/", "a.txt"), "/docs/a.txt");
    }
}
