use anyhow::{Context, Result, anyhow};
use openlist::OpenListClient;
use openlist::models::TaskInfo;

use crate::cli::TaskCommand;
use crate::commands::fs::human_size;
use crate::output;

pub async fn run(client: &OpenListClient, command: TaskCommand, json: bool) -> Result<()> {
    match command {
        TaskCommand::List { kind } => {
            let resp = client.task().undone(&kind).await?;
            emit_list(&resp, json)
        }
        TaskCommand::Done { kind } => {
            let resp = client.task().done(&kind).await?;
            emit_list(&resp, json)
        }
        TaskCommand::Info { id, kind } => {
            let resp = client.task().info(&kind, &id).await?;
            if json {
                return output::emit_json(&resp);
            }
            let task = resp.data.context("the server returned no task")?;
            print_task(&task);
            Ok(())
        }
        TaskCommand::Wait { id, kind, timeout } => {
            let task = wait_for(client, &kind, &id, timeout).await?;
            if json {
                return output::emit_json(&task);
            }
            finish(&task)
        }
        TaskCommand::Cancel { id, kind } => {
            let resp = client.task().cancel(&kind, &id).await?;
            if json {
                return output::emit_json(&resp);
            }
            output::emit_message(&resp)
        }
        TaskCommand::Retry { id, kind } => {
            let resp = client.task().retry(&kind, &id).await?;
            if json {
                return output::emit_json(&resp);
            }
            output::emit_message(&resp)
        }
    }
}

fn emit_list(resp: &openlist::models::ApiResponse<Vec<TaskInfo>>, json: bool) -> Result<()> {
    if json {
        return output::emit_json(resp);
    }
    let tasks = resp.data.clone().unwrap_or_default();
    if tasks.is_empty() {
        println!("(no tasks)");
        return Ok(());
    }
    let rows = tasks
        .iter()
        .map(|task| {
            vec![
                task.id.clone().unwrap_or_default(),
                task.state_name().to_string(),
                format!("{:.0}%", task.progress),
                human_size(Some(task.total_bytes)),
                task.name.clone().unwrap_or_default(),
                task.error.clone().unwrap_or_default(),
            ]
        })
        .collect();
    output::table(&["ID", "STATE", "PROGRESS", "SIZE", "NAME", "ERROR"], rows);
    Ok(())
}

fn print_task(task: &TaskInfo) {
    let rows = vec![
        output::field("id", &task.id),
        output::field("name", &task.name),
        output::field("state", &Some(task.state_name())),
        output::field("status", &task.status),
        output::field("progress", &Some(format!("{:.0}%", task.progress))),
        output::field("size", &Some(human_size(Some(task.total_bytes)))),
        output::field("start_time", &task.start_time),
        output::field("end_time", &task.end_time),
        output::field("error", &task.error),
    ];
    output::table(&["FIELD", "VALUE"], rows);
}

/// Poll until the task manager reports a terminal state.
pub(crate) async fn wait_for(
    client: &OpenListClient,
    kind: &str,
    id: &str,
    timeout_secs: u64,
) -> Result<TaskInfo> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let resp = client.task().info(kind, id).await?;
        let task = resp.data.context("the server returned no task")?;
        if task.is_terminal() {
            return Ok(task);
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(anyhow!(
                "task {id} did not finish within {timeout_secs}s (last state: {})",
                task.state_name()
            ));
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

/// Report a finished task, turning a failed or canceled state into an error.
fn finish(task: &TaskInfo) -> Result<()> {
    let id = task.id.clone().unwrap_or_default();
    if task.is_success() {
        println!(
            "Task {id} succeeded: {}",
            task.name.clone().unwrap_or_default()
        );
        return Ok(());
    }
    Err(anyhow!("task {id} {}", describe_failure(task)))
}

/// "failed: <reason>" / "canceled: no error reported", for CLI errors.
pub(crate) fn describe_failure(task: &TaskInfo) -> String {
    let reason = task.error.clone().unwrap_or_default();
    format!(
        "{}: {}",
        task.state_name(),
        if reason.is_empty() {
            "no error reported"
        } else {
            &reason
        }
    )
}
