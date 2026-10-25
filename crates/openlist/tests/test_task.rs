mod common;

use openlist::OpenListError;

fn task_json(id: &str, state: i32, error: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "upload report.txt to [/docs](/tianyi/docs)",
        "creator": "syndim",
        "creator_role": 2,
        "state": state,
        "status": "uploading",
        "progress": 42.5,
        "start_time": "2026-10-25T05:44:00+08:00",
        "end_time": null,
        "total_bytes": 1024,
        "error": error,
    })
}

#[tokio::test]
async fn test_task_undone_lists_tasks() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/task/upload/undone")
        .match_header("authorization", "test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            task_json("abc", 1, ""),
            task_json("def", 8, "boom"),
        ])))
        .create_async()
        .await;

    let resp = client.task().undone("upload").await.unwrap();
    let tasks = resp.data.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id.as_deref(), Some("abc"));
    assert_eq!(tasks[0].state_name(), "running");
    assert!(!tasks[0].is_terminal());
    assert_eq!(tasks[1].state_name(), "waiting-retry");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_task_done_lists_terminal_tasks() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("GET", "/api/task/upload/done")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!([
            task_json("abc", 2, ""),
            task_json("def", 7, "storage unavailable"),
        ])))
        .create_async()
        .await;

    let resp = client.task().done("upload").await.unwrap();
    let tasks = resp.data.unwrap();
    assert!(tasks[0].is_success());
    assert!(tasks[1].is_terminal());
    assert!(!tasks[1].is_success());
    assert_eq!(tasks[1].error.as_deref(), Some("storage unavailable"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_task_info_sends_the_task_id() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/task/upload/info")
        .match_query(mockito::Matcher::UrlEncoded("tid".into(), "abc".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(task_json("abc", 2, "")))
        .create_async()
        .await;

    let resp = client.task().info("upload", "abc").await.unwrap();
    let task = resp.data.unwrap();
    assert!(task.is_success());
    assert_eq!(task.total_bytes, 1024);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_task_cancel_posts_the_task_id() {
    let (mut server, client) = common::setup().await;
    let mock = server
        .mock("POST", "/api/task/upload/cancel")
        .match_query(mockito::Matcher::UrlEncoded("tid".into(), "abc".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(common::success_json(serde_json::json!(null)))
        .create_async()
        .await;

    let resp = client.task().cancel("upload", "abc").await.unwrap();
    assert_eq!(resp.code, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_task_rejects_unknown_kind_without_a_request() {
    let (_server, client) = common::setup().await;
    let err = client
        .task()
        .undone("upload/../../admin")
        .await
        .unwrap_err();
    assert!(matches!(err, OpenListError::InvalidTaskKind(_)), "{err}");
}
