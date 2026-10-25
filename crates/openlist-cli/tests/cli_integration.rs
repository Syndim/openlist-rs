//! End-to-end tests that exercise the `openlist` binary against a real
//! OpenList server.
//!
//! The server binary is not bundled with this repository. Point
//! `OPENLIST_TEST_SERVER_BIN` at an OpenList executable to run these tests;
//! when the variable is unset the test is skipped so `cargo test` still works
//! offline.

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use openlist::OpenListClient;
use openlist::models::Storage;
use serde_json::Value;

const SERVER_BIN_ENV: &str = "OPENLIST_TEST_SERVER_BIN";
const ADMIN_USERNAME: &str = "admin";
const ADMIN_PASSWORD: &str = "integration-test-password";

static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "openlist-rs-integration-{}-{nanos}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("failed to create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        if std::thread::panicking() && std::env::var_os("OPENLIST_TEST_KEEP_TEMP").is_some() {
            eprintln!("keeping failing test directory: {}", self.0.display());
            return;
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct TestServer {
    base_url: String,
    api_token: String,
    port: u16,
    data_dir: PathBuf,
    storage_dir: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    child: Child,
    _dir: TestDir,
}

impl TestServer {
    fn start() -> Option<Self> {
        let bin = std::env::var_os(SERVER_BIN_ENV).map(PathBuf::from)?;
        assert!(
            bin.is_file(),
            "{SERVER_BIN_ENV} does not point at a file: {}",
            bin.display()
        );

        let dir = TestDir::new();
        let data_dir = dir.path().join("server-data");
        let storage_dir = dir.path().join("storage");
        fs::create_dir_all(&data_dir).expect("failed to create server data directory");
        fs::create_dir_all(&storage_dir).expect("failed to create storage directory");

        let (port, config) = reserve_port();
        let config_path = data_dir.join("config.json");
        fs::write(
            &config_path,
            serde_json::to_vec_pretty(&config).expect("failed to serialize config"),
        )
        .expect("failed to write server config");

        // Initialise the database before starting the HTTP server. A fresh
        // OpenList instance only creates the admin user when
        // OPENLIST_ADMIN_PASSWORD is set (otherwise the web setup wizard is
        // expected to do it). The same invocation prints the static admin
        // token, which the API accepts as an API key in the Authorization
        // header.
        let output = Command::new(&bin)
            .arg("--data")
            .arg(&data_dir)
            .args(["admin", "token"])
            .env("OPENLIST_ADMIN_PASSWORD", ADMIN_PASSWORD)
            .output()
            .expect("failed to run the OpenList admin command");
        assert!(
            output.status.success(),
            "could not initialise the OpenList server:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        let api_token = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("Admin token: "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!(
                    "could not read the admin token from:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                )
            });

        // OpenList rewrites config.json from `conf.DefaultConfig` before
        // merging the file, so the port chosen above is lost after this step.
        // Restore it so the server binds an OS-assigned port.
        let mut config: Value =
            serde_json::from_slice(&fs::read(&config_path).expect("failed to read server config"))
                .expect("failed to parse server config");
        config["scheme"] = serde_json::json!({ "address": "127.0.0.1", "http_port": port });
        fs::write(
            &config_path,
            serde_json::to_vec_pretty(&config).expect("failed to serialize config"),
        )
        .expect("failed to rewrite server config");

        let stdout_path = dir.path().join("server.stdout");
        let stderr_path = dir.path().join("server.stderr");
        let stdout = fs::File::create(&stdout_path).expect("failed to create server stdout log");
        let stderr = fs::File::create(&stderr_path).expect("failed to create server stderr log");

        let child = Command::new(&bin)
            .arg("--data")
            .arg(&data_dir)
            .arg("server")
            .env("OPENLIST_ADMIN_PASSWORD", ADMIN_PASSWORD)
            // Disable every bundled offline-download tool so startup does not
            // probe localhost ports that tests running in parallel may own.
            .env("OPENLIST_OFFLINE_DOWNLOAD_TOOLS_ARIA2_ENABLE", "false")
            .env(
                "OPENLIST_OFFLINE_DOWNLOAD_TOOLS_QBITTORRENT_ENABLE",
                "false",
            )
            .env(
                "OPENLIST_OFFLINE_DOWNLOAD_TOOLS_TRANSMISSION_ENABLE",
                "false",
            )
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("failed to start the OpenList server");

        Some(Self {
            api_token,
            port,
            data_dir,
            storage_dir,
            stdout_path,
            stderr_path,
            child,
            base_url: String::new(),
            _dir: dir,
        })
    }

    async fn wait_until_ready(&mut self) {
        self.base_url = format!("http://127.0.0.1:{}", self.port);
        let client = OpenListClient::new(self.base_url.clone());
        for _ in 0..150 {
            if client.public().settings().await.is_ok() {
                return;
            }
            if let Some(status) = self.child.try_wait().expect("failed to poll the server") {
                panic!("OpenList exited with {status}\n{}", self.logs());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        panic!(
            "OpenList did not become ready at {}\n{}",
            self.base_url,
            self.logs()
        );
    }

    async fn launch() -> Option<Self> {
        let mut server = Self::start()?;
        server.wait_until_ready().await;
        server.create_storage().await;
        Some(server)
    }

    /// A fresh OpenList instance has no storage mounts. Provision a local
    /// storage at "/" so the filesystem API has somewhere to write.
    async fn create_storage(&self) {
        let admin = OpenListClient::new(&self.base_url).with_token(self.api_token.clone());
        let storage = Storage {
            mount_path: Some("/".into()),
            order: Some(0),
            driver: Some("Local".into()),
            cache_expiration: Some(30),
            status: Some(String::new()),
            addition: Some(
                serde_json::json!({
                    "root_folder_path": self.storage_dir,
                    "thumbnail": false,
                })
                .to_string(),
            ),
            remark: Some("openlist-rs integration test".into()),
            ..Storage::default()
        };
        admin
            .admin()
            .create_storage(&storage)
            .await
            .expect("failed to create the local storage mount");
    }

    /// Build one authenticated CLI runner per supported auth mode: a cached
    /// login JWT and the static admin token used as an API key.
    fn auth_runners(&self, base: &Path) -> Vec<AuthRunner> {
        let jwt = CliRunner::new(&self.base_url, base.join("home-jwt"));
        let login = jwt.run_json(&[
            "--json",
            "login",
            "-u",
            ADMIN_USERNAME,
            "-p",
            ADMIN_PASSWORD,
        ]);
        let token = login["data"]["token"]
            .as_str()
            .expect("login did not return a token");
        assert!(!token.is_empty(), "login returned an empty token");

        let api = CliRunner::new(&self.base_url, base.join("home-api"))
            .with_token(self.api_token.clone());

        let runners = vec![
            AuthRunner {
                mode: "jwt",
                cli: jwt,
            },
            AuthRunner {
                mode: "api-key",
                cli: api,
            },
        ];

        // Both modes must authenticate as the admin user before running the
        // operation under test.
        for runner in &runners {
            let me = runner.cli.run_json(&["--json", "me"]);
            assert_eq!(
                me["data"]["username"], ADMIN_USERNAME,
                "{} runner is not authenticated as admin: {me}",
                runner.mode
            );
        }

        runners
    }

    fn logs(&self) -> String {
        let mut logs = String::new();
        for (label, path) in [
            ("stdout", &self.stdout_path),
            ("stderr", &self.stderr_path),
            ("server log", &self.data_dir.join("log").join("log.log")),
        ] {
            let contents = fs::read_to_string(path).unwrap_or_default();
            if !contents.is_empty() {
                logs.push_str(&format!(
                    "--- {label} ({}) ---\n{contents}\n",
                    path.display()
                ));
            }
        }
        logs
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct CliRunner {
    bin: PathBuf,
    server: String,
    home: PathBuf,
    token: Option<String>,
}

impl CliRunner {
    fn new(server: &str, home: PathBuf) -> Self {
        fs::create_dir_all(&home).expect("failed to create isolated home directory");
        Self {
            bin: PathBuf::from(env!("CARGO_BIN_EXE_openlist")),
            server: server.to_string(),
            home,
            token: None,
        }
    }

    fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.bin);
        command
            .args(args)
            .env("OPENLIST_SERVER", &self.server)
            // Keep credentials in the temporary home so the developer's real
            // configuration is never touched.
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("APPDATA", &self.home);
        match &self.token {
            Some(token) => {
                command.env("OPENLIST_TOKEN", token);
            }
            None => {
                command.env_remove("OPENLIST_TOKEN");
            }
        }
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args)
            .output()
            .expect("failed to run the openlist CLI")
    }

    fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        self.command(args)
            .current_dir(dir)
            .output()
            .expect("failed to run the openlist CLI")
    }

    fn run_ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "openlist {args:?} failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout).expect("CLI output was not valid UTF-8")
    }

    fn run_ok_in(&self, dir: &Path, args: &[&str]) -> String {
        let output = self.run_in(dir, args);
        assert!(
            output.status.success(),
            "openlist {args:?} failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout).expect("CLI output was not valid UTF-8")
    }

    fn run_json(&self, args: &[&str]) -> Value {
        let stdout = self.run_ok(args);
        serde_json::from_str(&stdout)
            .unwrap_or_else(|err| panic!("openlist {args:?} did not produce JSON: {err}\n{stdout}"))
    }
}

/// Bind an ephemeral port, release it and return the port together with the
/// OpenList `scheme` config that uses it. The OS avoids handing out ports that
/// were released moments ago, so the server can bind it right after.
fn reserve_port() -> (u16, Value) {
    let port = TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind an ephemeral port")
        .local_addr()
        .expect("failed to read the ephemeral port")
        .port();
    let config = serde_json::json!({
        "scheme": {
            "address": "127.0.0.1",
            "http_port": port,
        }
    });
    (port, config)
}

/// One authenticated CLI invocation context used to repeat a test under a
/// specific credential type.
struct AuthRunner {
    mode: &'static str,
    cli: CliRunner,
}

const CONTENT: &[u8] = b"hello from the openlist-rs integration test\n";

fn fixture(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    fs::create_dir_all(dir).expect("failed to create the fixture directory");
    let path = dir.join(name);
    fs::write(&path, content).expect("failed to write the fixture");
    path
}

fn find_user_id(listing: &Value, username: &str) -> Option<i64> {
    listing["data"]["content"]
        .as_array()?
        .iter()
        .find(|user| user["username"].as_str() == Some(username))
        .and_then(|user| user["id"].as_i64())
}

fn share_ids(listing: &Value) -> Vec<String> {
    listing["data"]["content"]
        .as_array()
        .map(|shares| {
            shares
                .iter()
                .filter_map(|share| share["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn auth_login_logout_and_api_token() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let work = server._dir.path();

    // JWT: login caches the token; logout clears it.
    let jwt = CliRunner::new(&server.base_url, work.join("jwt-home"));
    let login = jwt.run_json(&[
        "--json",
        "login",
        "-u",
        ADMIN_USERNAME,
        "-p",
        ADMIN_PASSWORD,
    ]);
    let token = login["data"]["token"]
        .as_str()
        .expect("login did not return a token");
    assert!(!token.is_empty(), "login returned an empty token");
    assert_eq!(
        jwt.run_json(&["--json", "me"])["data"]["username"],
        ADMIN_USERNAME
    );

    jwt.run_ok(&["logout"]);
    let after_logout = jwt.run(&["--json", "me"]);
    assert!(
        !after_logout.status.success(),
        "me should fail after logout, stdout:\n{}",
        String::from_utf8_lossy(&after_logout.stdout)
    );

    // API key: the static admin token from a credential-free home.
    let api = CliRunner::new(&server.base_url, work.join("api-home"))
        .with_token(server.api_token.clone());
    assert_eq!(
        api.run_json(&["--json", "me"])["data"]["username"],
        ADMIN_USERNAME
    );
}

#[tokio::test]
async fn upload_list_stat_and_download() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let base = server._dir.path().join("upload");

    for runner in server.auth_runners(&base) {
        let work = base.join(runner.mode);
        let root = format!("/upload-{}", runner.mode);
        let cli = &runner.cli;

        cli.run_ok(&["mkdir", &root]);

        let hello = fixture(&work, "hello.txt", CONTENT);
        cli.run_ok(&[
            "upload",
            hello.to_str().expect("non UTF-8 upload path"),
            &root,
            "--name",
            "hello.txt",
        ]);

        // Without --name the remote name defaults to the local file name.
        let default_upload = fixture(&work, "default-upload.txt", b"default name\n");
        cli.run_ok(&[
            "upload",
            default_upload.to_str().expect("non UTF-8 upload path"),
            &root,
        ]);

        let listing = cli.run_json(&["ls", &root, "--json", "--refresh"]);
        let entries = listing["data"]["content"]
            .as_array()
            .expect("ls did not return a content array");
        assert_eq!(entries.len(), 2, "unexpected listing: {listing}");
        assert!(
            entries.iter().any(|entry| entry["name"] == "hello.txt"),
            "missing hello.txt: {listing}"
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry["name"] == "default-upload.txt"),
            "missing default-upload.txt: {listing}"
        );

        let page = cli.run_json(&["ls", &root, "--json", "--page", "1", "--per-page", "1"]);
        assert_eq!(page["data"]["total"].as_i64(), Some(2));
        assert_eq!(
            page["data"]["content"]
                .as_array()
                .expect("paginated ls did not return a content array")
                .len(),
            1
        );

        let hello_remote = format!("{root}/hello.txt");
        let stat = cli.run_json(&["stat", &hello_remote, "--json"]);
        assert_eq!(stat["data"]["name"], "hello.txt");
        assert_eq!(stat["data"]["size"].as_i64(), Some(CONTENT.len() as i64));

        // Explicit download destination, including missing parent directories.
        let downloaded = work.join("downloads/nested/hello.txt");
        cli.run_ok(&[
            "download",
            &hello_remote,
            downloaded.to_str().expect("non UTF-8 download path"),
        ]);
        assert_eq!(fs::read(&downloaded).unwrap(), CONTENT);

        // Default download destination: the remote file name in the cwd.
        let default_dir = work.join("download-default");
        fs::create_dir_all(&default_dir).expect("failed to create download directory");
        cli.run_ok_in(&default_dir, &["download", &hello_remote]);
        assert_eq!(fs::read(default_dir.join("hello.txt")).unwrap(), CONTENT);

        cli.run_ok(&["rm", &root, "hello.txt", "-y"]);
        cli.run_ok(&["rm", &root, "default-upload.txt", "-y"]);
        cli.run_ok(&["rm", "/", root.trim_start_matches('/'), "-y"]);
    }
}

#[tokio::test]
async fn rename_copy_move_and_remove() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let base = server._dir.path().join("mutations");

    for runner in server.auth_runners(&base) {
        let work = base.join(runner.mode);
        let root = format!("/mutations-{}", runner.mode);
        let cli = &runner.cli;

        cli.run_ok(&["mkdir", &root]);
        let upload = fixture(&work, "hello.txt", CONTENT);
        cli.run_ok(&[
            "upload",
            upload.to_str().expect("non UTF-8 upload path"),
            &root,
            "--name",
            "hello.txt",
        ]);

        let hello = format!("{root}/hello.txt");
        cli.run_ok(&["rename", &hello, "renamed.txt"]);

        let copy_dir = format!("{root}/copy");
        cli.run_ok(&["mkdir", &copy_dir]);
        cli.run_ok(&["cp", &root, &copy_dir, "renamed.txt"]);

        let copied_remote = format!("{copy_dir}/renamed.txt");
        let copied = work.join("copied.txt");
        cli.run_ok(&[
            "download",
            &copied_remote,
            copied.to_str().expect("non UTF-8 copy path"),
        ]);
        assert_eq!(fs::read(&copied).unwrap(), CONTENT);

        let moved_dir = format!("{root}/moved");
        cli.run_ok(&["mkdir", &moved_dir]);
        cli.run_ok(&["mv", &root, &moved_dir, "renamed.txt"]);

        let moved_remote = format!("{moved_dir}/renamed.txt");
        let moved = work.join("moved.txt");
        cli.run_ok(&[
            "download",
            &moved_remote,
            moved.to_str().expect("non UTF-8 move path"),
        ]);
        assert_eq!(fs::read(&moved).unwrap(), CONTENT);

        cli.run_ok(&["rm", &copy_dir, "renamed.txt", "-y"]);
        cli.run_ok(&["rm", &moved_dir, "renamed.txt", "-y"]);
        cli.run_ok(&["rm", "/", root.trim_start_matches('/'), "-y"]);

        let listing = cli.run_json(&["ls", "/", "--json"]);
        let entries = listing["data"]["content"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(
            entries
                .iter()
                .all(|entry| entry["name"].as_str() != Some(root.trim_start_matches('/'))),
            "{root} should have been removed: {listing}"
        );
    }
}

#[tokio::test]
async fn share_links_lifecycle() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let base = server._dir.path().join("shares");

    for runner in server.auth_runners(&base) {
        let work = base.join(runner.mode);
        let root = format!("/shares-{}", runner.mode);
        let cli = &runner.cli;

        cli.run_ok(&["mkdir", &root]);
        let upload = fixture(&work, "shared.txt", CONTENT);
        cli.run_ok(&[
            "upload",
            upload.to_str().expect("non UTF-8 upload path"),
            &root,
            "--name",
            "shared.txt",
        ]);
        let remote = format!("{root}/shared.txt");

        let created = cli.run_json(&["share", "create", &remote, "--expiration", "0", "--json"]);
        let id = created["data"]["id"]
            .as_str()
            .expect("share create did not return an id")
            .to_string();
        assert!(!id.is_empty(), "share create returned an empty id");

        let listed = cli.run_json(&["share", "list", "--json"]);
        assert!(
            share_ids(&listed).contains(&id),
            "created share {id} missing from list: {listed}"
        );

        let got = cli.run_json(&["share", "get", &id, "--json"]);
        assert_eq!(got["data"]["id"], id);
        assert_eq!(got["data"]["files"][0], remote);

        cli.run_ok(&[
            "share",
            "update",
            &id,
            "--password",
            "updated-password",
            "--expiration",
            "0",
        ]);
        let updated = cli.run_json(&["share", "get", &id, "--json"]);
        assert_eq!(updated["data"]["disabled"], false);

        cli.run_ok(&["share", "disable", &id]);
        let disabled = cli.run_json(&["share", "get", &id, "--json"]);
        assert_eq!(disabled["data"]["disabled"], true);

        cli.run_ok(&["share", "enable", &id]);
        let enabled = cli.run_json(&["share", "get", &id, "--json"]);
        assert_eq!(enabled["data"]["disabled"], false);

        cli.run_ok(&["share", "delete", &id]);
        let after = cli.run_json(&["share", "list", "--json"]);
        assert!(
            !share_ids(&after).contains(&id),
            "deleted share {id} still in list: {after}"
        );

        cli.run_ok(&["rm", &root, "shared.txt", "-y"]);
        cli.run_ok(&["rm", "/", root.trim_start_matches('/'), "-y"]);
    }
}

#[tokio::test]
async fn admin_users_lifecycle() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let base = server._dir.path().join("users");

    for runner in server.auth_runners(&base) {
        let username = format!("it{}", runner.mode.replace('-', ""));
        let cli = &runner.cli;

        cli.run_ok(&[
            "admin",
            "users",
            "create",
            &username,
            "-p",
            "user-password",
            "--role",
            "0",
        ]);

        let listed = cli.run_json(&["admin", "users", "list", "--json"]);
        let id = find_user_id(&listed, &username)
            .unwrap_or_else(|| panic!("created user {username} missing from list: {listed}"));

        let got = cli.run_json(&["admin", "users", "get", &id.to_string(), "--json"]);
        assert_eq!(got["data"]["username"], username);

        cli.run_ok(&["admin", "users", "delete", &id.to_string()]);
        let after = cli.run_json(&["admin", "users", "list", "--json"]);
        assert!(
            find_user_id(&after, &username).is_none(),
            "deleted user {username} still in list: {after}"
        );
    }
}

#[tokio::test]
async fn admin_storages_lifecycle() {
    let Some(server) = TestServer::launch().await else {
        eprintln!("skipping integration test: set {SERVER_BIN_ENV} to an OpenList server binary");
        return;
    };
    let base = server._dir.path().join("storages");

    for runner in server.auth_runners(&base) {
        let cli = &runner.cli;

        let listed = cli.run_json(&["admin", "storages", "list", "--json"]);
        let id = listed["data"]["content"]
            .as_array()
            .and_then(|storages| storages.first())
            .and_then(|storage| storage["id"].as_i64())
            .expect("no storage returned by admin storages list");
        let id_arg = id.to_string();

        let got = cli.run_json(&["admin", "storages", "get", &id_arg, "--json"]);
        assert_eq!(got["data"]["driver"], "Local");
        assert_eq!(got["data"]["mount_path"], "/");

        cli.run_ok(&["admin", "storages", "disable", &id_arg]);
        let disabled = cli.run_json(&["admin", "storages", "get", &id_arg, "--json"]);
        assert_eq!(disabled["data"]["disabled"], true);

        cli.run_ok(&["admin", "storages", "enable", &id_arg]);
        let enabled = cli.run_json(&["admin", "storages", "get", &id_arg, "--json"]);
        assert_eq!(enabled["data"]["disabled"], false);
    }
}
