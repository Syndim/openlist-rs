# OpenList Rust SDK

A typed, async Rust client for the [OpenList](https://github.com/OpenListTeam/OpenList) API, plus a command-line interface built on the same library.

> **Unofficial client.** This project is an independent, community-maintained
> client and is not affiliated with, endorsed by, or supported by the
> [OpenList](https://github.com/OpenListTeam/OpenList) project. It only talks to
> an OpenList server over its HTTP API. Please report bugs in this SDK or CLI
> here, and problems with the server itself upstream.

[![Release](https://github.com/Syndim/openlist-rs/actions/workflows/release.yml/badge.svg)](https://github.com/Syndim/openlist-rs/actions/workflows/release.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)

The repository, [Syndim/openlist-rs](https://github.com/Syndim/openlist-rs), is a Cargo workspace with two crates:

| Crate | Kind | Description |
| --- | --- | --- |
| [`openlist`](crates/openlist) | library | Async, typed SDK for the OpenList HTTP API |
| [`openlist-cli`](crates/openlist-cli) | binary (`openlist`) | Human-friendly CLI and JSON output for the SDK |

---

## Table of contents

- [Features](#features)
- [Requirements](#requirements)
- [Workspace layout](#workspace-layout)
- [CLI](#cli)
  - [Install](#install)
  - [Quick start](#quick-start)
  - [Global options](#global-options)
  - [Commands](#commands)
  - [Examples](#examples)
  - [Credentials and configuration](#credentials-and-configuration)
  - [JSON output](#json-output)
- [Library](#library)
  - [Add the dependency](#add-the-dependency)
  - [Library quick start](#library-quick-start)
  - [Error handling](#error-handling)
  - [API coverage](#api-coverage)
- [Development](#development)
- [Release process](#release-process)
- [License](#license)

---

## Features

- Typed request and response models for the OpenList API
- Async/await API built on `reqwest` + `tokio`
- HTTPS through rustls; no OpenSSL required
- Authentication: password, password hash, LDAP, 2FA, SSO and WebAuthn endpoints
- Filesystem operations: list, stat, search, mkdir, rename, batch/regex rename, move, copy, remove, upload, offline download and archive inspection
- Administration: users, storage mounts, drivers, settings, metadata, index and scan
- Share-link management: create, update, enable/disable and delete
- CLI output as aligned tables or raw JSON (`--json`)
- Local credential storage with `0600` permissions on Unix

## Requirements

- Rust **1.85** or newer (the workspace uses edition 2024)
- A running and reachable OpenList server
- For library users: a Tokio runtime (the crate uses `tokio` internally)

## Workspace layout

```text
.
├── Cargo.toml                  # virtual workspace manifest
├── Cargo.lock
├── crates/
│   ├── openlist/               # library crate
│   │   ├── src/
│   │   └── tests/              # mockito-backed integration tests
│   └── openlist-cli/           # binary crate
│       └── src/
└── .github/workflows/
    ├── integration.yml         # all tests incl. end-to-end; reused by release
    └── release.yml             # tag-driven release pipeline
```

---

## CLI

The binary is named `openlist`.

### Install

Prebuilt binaries for Linux, macOS and Windows are attached to each [release](https://github.com/Syndim/openlist-rs/releases). Install directly from GitHub with Cargo:

```bash
cargo install --git https://github.com/Syndim/openlist-rs.git --locked openlist-cli
```

Or from a checkout:

```bash
git clone https://github.com/Syndim/openlist-rs.git
cd openlist-rs
cargo install --path crates/openlist-cli --locked
```

Build without installing:

```bash
cargo build --release -p openlist-cli
./target/release/openlist --help
```

### Quick start

```bash
# Point the CLI at your server (defaults to http://localhost:5244)
export OPENLIST_SERVER=http://localhost:5244

# Log in; the resulting token is cached locally
openlist login -u admin -p 'your-password'

# Check the session and browse files
openlist me
openlist ls /
```

### Global options

Global flags may appear before or after the subcommand.

| Option | Environment variable | Default | Description |
| --- | --- | --- | --- |
| `--server <URL>` | `OPENLIST_SERVER` | `http://localhost:5244` | OpenList server base URL |
| `--token <TOKEN>` | `OPENLIST_TOKEN` | stored token | API token; overrides the cached credential |
| `--json` | — | off | Print the raw JSON response instead of a table |
| `-v, --verbose` | — | off | Print request/response details to stderr |
| `-h, --help` | — | — | Show help |
| `-V, --version` | — | — | Show version |

### Commands

#### Authentication and account

| Command | Description |
| --- | --- |
| `openlist login -u <USER> -p <PASSWORD> [--otp-code <CODE>]` | Authenticate and cache the token |
| `openlist logout` | Invalidate the session and clear the cached token |
| `openlist me` | Show the current user |

#### Filesystem

| Command | Description |
| --- | --- |
| `openlist ls [PATH] [--refresh] [--page N] [--per-page N]` | List a directory (defaults to `/`) |
| `openlist stat <PATH>` | Show metadata for one entry |
| `openlist search <PARENT> <KEYWORDS> [--scope N]` | Search below a parent path |
| `openlist mkdir <PATH>` | Create a directory |
| `openlist rename <PATH> <NEW_NAME>` | Rename an entry in place |
| `openlist mv <SRC_DIR> <DST_DIR> <NAMES>...` | Move entries |
| `openlist cp <SRC_DIR> <DST_DIR> <NAMES>...` | Copy entries |
| `openlist rm <DIR> <NAMES>... [-y]` | Remove entries; prompts unless `-y/--yes` |
| `openlist upload <LOCAL> <REMOTE_DIR> [--name <NAME>] [--as-task] [--wait] [--timeout SECONDS]` | Upload one local file; `--as-task` returns after the server spools the body, `--wait` polls the resulting task and exits non-zero if it fails |
| `openlist download <REMOTE_PATH> [LOCAL_PATH]` | Download a remote file |

#### Tasks

Every `--kind` below accepts `upload` (default), `copy`, `move`, `offline_download`, `offline_download_transfer`, `decompress` or `decompress_upload`.

| Command | Description |
| --- | --- |
| `openlist task list [--kind KIND]` | List unfinished tasks (pending, running, retrying) |
| `openlist task done [--kind KIND]` | List finished tasks |
| `openlist task info <ID> [--kind KIND]` | Show one task: state, progress, size, error |
| `openlist task wait <ID> [--kind KIND] [--timeout SECONDS]` | Poll until the task finishes; exits non-zero when it failed or was canceled |
| `openlist task cancel <ID> [--kind KIND]` | Cancel a task |
| `openlist task retry <ID> [--kind KIND]` | Retry a failed task |

#### Share links

| Command | Description |
| --- | --- |
| `openlist share list [--page N] [--per-page N]` | List share links |
| `openlist share get <ID>` | Show one share link |
| `openlist share create <PATHS>... [--password P] [--expiration E]` | Create a share link (`0` means never, `30m`/`12h`/`7d` are relative) |
| `openlist share update <ID> [--path P] [--password P] [--expiration E] [--disable BOOL]` | Update a share link |
| `openlist share delete <ID>` | Delete a share link |
| `openlist share enable <ID>` | Enable a share link |
| `openlist share disable <ID>` | Disable a share link |

#### Administration

| Command | Description |
| --- | --- |
| `openlist admin users list [--page N] [--per-page N]` | List users |
| `openlist admin users get <ID>` | Show a user |
| `openlist admin users create <USERNAME> -p <PASSWORD> [--base-path P] [--role N] [--permission N] [--disabled]` | Create a user |
| `openlist admin users delete <ID>` | Delete a user |
| `openlist admin storages list [--page N] [--per-page N]` | List storage mounts |
| `openlist admin storages get <ID>` | Show a storage mount |
| `openlist admin storages enable <ID>` | Enable a storage mount |
| `openlist admin storages disable <ID>` | Disable a storage mount |

### Examples

```bash
# Authenticate against a specific server
openlist --server http://nas.local:5244 login -u admin -p 's3cret'

# Browse
openlist ls /
openlist ls /media --json
openlist stat /media/movie.mkv

# Search
openlist search /backups report

# Organise files
openlist mkdir /archive
openlist mv /downloads /archive movie.mkv
openlist cp /archive /backup movie.mkv
openlist rm /archive movie.mkv -y

# Upload and download
openlist upload ./report.pdf /docs --name report-2026.pdf
# A storage slower than the reverse-proxy timeout: let the server queue it
# A storage slower than the reverse-proxy timeout: let the server queue it,
# print the task id, and poll it to completion in the same run
openlist upload ./release.apk /packages --wait --timeout 1800

# ...or queue it and follow up in a later step
openlist upload ./release.apk /packages --as-task
openlist task info <task-id>
openlist task wait <task-id> --timeout 1800
openlist download /docs/report-2026.pdf ./report.pdf

# Share links
openlist share create /docs/report-2026.pdf --password hntr2 --expiration 0
openlist share list
openlist share disable <share-id>

# Administration
openlist admin users list --per-page 25
openlist admin storages list
openlist admin storages disable 3
```

### Credentials and configuration

`login` stores the server URL and token in a per-user configuration file:

| Platform | Path |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/openlist-cli/config.json` (usually `~/.config/openlist-cli/config.json`) |
| macOS | `~/Library/Application Support/openlist-cli/config.json` |
| Windows | `%APPDATA%\openlist-cli\config.json` |

Resolution order:

1. `--server` / `OPENLIST_SERVER`
2. Server saved by a previous `login`
3. `http://localhost:5244`

The token resolves in this order:

1. `--token` / `OPENLIST_TOKEN`
2. Token saved by a previous `login`

On Unix the configuration file is written with mode `0600`. For automation, prefer `OPENLIST_TOKEN` over passing credentials on the command line.

### JSON output

Pass `--json` (globally or after the subcommand) to print the API envelope:

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "content": [],
    "total": 0
  }
}
```

This is useful for scripting with `jq`:

```bash
openlist ls /docs --json | jq -r '.data.content[].name'
```

---

## Library

### Add the dependency

From this workspace:

```toml
[dependencies]
openlist = { path = "crates/openlist" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

From another repository, use a path or Git dependency:

```toml
[dependencies]
# openlist = { path = "../openlist/crates/openlist" }
openlist = { git = "https://github.com/Syndim/openlist-rs.git" }
```

### Library quick start

```rust
use openlist::models::{FsListRequest, LoginRequest};
use openlist::OpenListClient;

#[tokio::main]
async fn main() -> openlist::Result<()> {
    let mut client = OpenListClient::new("http://localhost:5244");

    // Password login. Use login_hash / login_ldap for other flows.
    let login = client
        .auth()
        .login(&LoginRequest {
            username: "admin".into(),
            password: "password".into(),
            otp_code: None,
        })
        .await?;

    if let Some(data) = login.data {
        client.set_token(data.token);
    }

    let listing = client
        .fs()
        .list(&FsListRequest {
            path: Some("/".into()),
            password: None,
            refresh: None,
            page: None,
            per_page: None,
        })
        .await?;

    if let Some(data) = listing.data {
        for entry in data.content.unwrap_or_default() {
            let kind = if entry.is_dir.unwrap_or(false) { "dir " } else { "file" };
            println!("{kind} {}", entry.name.unwrap_or_default());
        }
    }

    Ok(())
}
```

If you already have a token, skip the login call:

```rust
use openlist::OpenListClient;

let client = OpenListClient::new("http://localhost:5244")
    .with_token(std::env::var("OPENLIST_TOKEN").unwrap());

let resp = client.user().me().await?;
println!("{resp:?}");
```

Enable request/response logging for debugging:

```rust
let client = OpenListClient::new("http://localhost:5244")
    .with_token(token)
    .with_verbose(true);
```

Every API group is reached from the client:

| Accessor | Covers |
| --- | --- |
| `client.auth()` | Login (password, hash, LDAP), logout, 2FA, SSO and WebAuthn |
| `client.user()` | Current user, password update, SSH keys |
| `client.fs()` | Listing, stat, search, directories, rename, move/copy, remove, upload, offline download, archives |
| `client.admin()` | Users, storage mounts, drivers, settings, metadata, index and scan |
| `client.public()` | Public settings, offline-download tools, archive extensions |
| `client.sharing()` | Share-link CRUD and enable/disable |
| `client.task()` | Background tasks: `undone`, `done`, `info`, `cancel`, `retry` |

### Error handling

All methods return `openlist::Result<T>`, where `openlist::OpenListError` distinguishes transport, API and serialization failures:

```rust
use openlist::OpenListError;

match client.user().me().await {
    Ok(resp) => println!("{resp:?}"),
    Err(OpenListError::Api { code, message }) => {
        eprintln!("OpenList rejected the request ({code}): {message}");
    }
    Err(err) => eprintln!("request failed: {err}"),
}
```

When a reverse proxy, CDN or gateway answers instead of OpenList, the body is
not JSON. Those failures are reported as `OpenListError::NonJsonResponse` with
the HTTP status, content type and a body preview, for example:

```
HTTP 504 (text/plain; charset=UTF-8), body was not the OpenList JSON API response: error code: 504; the request did not get an answer from OpenList (gateway or origin timeout)
```

A `504`/`524` after roughly a minute means the proxy gave up waiting for the
storage write: the request reached OpenList, but the mounted storage did not
answer. `upload --as-task` sidesteps that timeout by returning as soon as the
server has spooled the body.

### API coverage

- **Auth**: `login`, `login_hash`, `login_ldap`, `logout`, `generate_2fa`, `verify_2fa`, `sso_login_redirect`, `sso_callback`, WebAuthn login/registration/credentials
- **User**: `me`, `update_me`, `list_ssh_keys`, `add_ssh_key`, `delete_ssh_key`
- **Tasks**: `undone`, `done`, `info`, `cancel`, `retry` for the upload, copy, move, offline-download, transfer and decompress task managers
- **Filesystem**: `list`, `get`, `download`, `search`, `dirs`, `other`, `mkdir`, `rename`, `batch_rename`, `regex_rename`, `move_files`, `recursive_move`, `copy`, `remove`, `remove_empty_directory`, `put`, `form_upload`, `add_offline_download`, `archive_decompress`, `archive_meta`, `archive_list`
- **Admin**: user CRUD, 2FA cancellation, user cache and SSH keys; storage CRUD, enable/disable and reload; drivers; settings; metadata; index build/update/stop/clear/progress; scan start/stop/progress
- **Public**: `settings`, `offline_download_tools`, `archive_extensions`
- **Sharing**: `list`, `get`, `create`, `update`, `delete`, `enable`, `disable`

---

## Development

```bash
# Build everything
cargo build --workspace

# Run the whole test suite (mock server, no OpenList instance required)
cargo test --workspace

# Library tests only
cargo test -p openlist

# Exercise the CLI
cargo run -p openlist-cli -- --help

# Formatting and lints
cargo fmt --all
cargo clippy --workspace --all-targets
```

The tests under `crates/openlist/tests` use [`mockito`](https://docs.rs/mockito) and run entirely offline.

The end-to-end suite in `crates/openlist-cli/tests/cli_integration.rs` starts a real OpenList server (one per test, in parallel) and drives the CLI through every command:

| Test | Covers |
| --- | --- |
| `auth_login_logout_and_api_token` | `login`, `logout`, `me` |
| `upload_list_stat_and_download` | `mkdir`, `upload` (with and without `--name`), `ls` (pagination, `--refresh`), `stat`, `download` (explicit and default destination) |
| `rename_copy_move_and_remove` | `rename`, `cp`, `mv`, `rm` |
| `share_links_lifecycle` | `share list/get/create/update/enable/disable/delete` |
| `admin_users_lifecycle` | `admin users list/get/create/delete` |
| `admin_storages_lifecycle` | `admin storages list/get/enable/disable` |

Every operation runs twice: once with the cached login JWT and once with the static admin token passed via `OPENLIST_TOKEN` (the API-key path), each from an isolated config directory. It needs an OpenList server binary and is skipped when `OPENLIST_TEST_SERVER_BIN` is unset:

The easiest path is [`just`](https://github.com/casey/just):

```bash
just check            # fmt --check + clippy + offline tests
just test             # offline tests only
just e2e              # download the latest server, run the suite, delete it again
just integration      # end-to-end suite (keeps the downloaded server)
just integration share_links   # a single integration test
just server-remove    # delete the downloaded server binary
just cli ls /         # run the CLI from source
just server           # throw-away server on 127.0.0.1:5244 (Ctrl-C to stop)
```

The steps behind `just integration`, if you prefer to drive it yourself:

```bash
# 1. Fetch the latest server binary for your platform.
#    Asset names: openlist-darwin-arm64.tar.gz, openlist-linux-amd64.tar.gz, ...
#    (`just server-install` does this for you.)
curl -fsSL -o /tmp/openlist.tar.gz \
  "https://github.com/OpenListTeam/OpenList/releases/latest/download/openlist-darwin-arm64.tar.gz"
mkdir -p ~/.local/bin && tar -xzf /tmp/openlist.tar.gz -C ~/.local/bin
chmod +x ~/.local/bin/openlist

# 2. Point the suite at it, either once per shell:
export OPENLIST_TEST_SERVER_BIN="$HOME/.local/bin/openlist"

# ...or persistently with direnv (.envrc is git-ignored):
echo 'export OPENLIST_TEST_SERVER_BIN="$HOME/.local/bin/openlist"' > .envrc
direnv allow

# 3. Run the suite.
cargo test -p openlist-cli --test cli_integration -- --nocapture
```

The tests never touch `~/.config/openlist-cli`: each server and CLI invocation runs with its own temporary `HOME`/`XDG_CONFIG_HOME`.

Set `OPENLIST_TEST_KEEP_TEMP=1` to keep the temporary server directory and logs when the test fails.

The [Integration workflow](.github/workflows/integration.yml) runs the **whole** suite — `cargo test --workspace` — with `OPENLIST_TEST_SERVER_BIN` set, so the offline tests and the end-to-end tests run together on every branch push and pull request. Tag pushes are deliberately excluded there: the Release workflow calls this workflow as a job (`workflow_call`) instead, so each release runs the tests exactly once and cannot publish unless they pass.

## Release process

Pushing a tag that matches `v*` triggers [`.github/workflows/release.yml`](.github/workflows/release.yml):

1. Verifies that the tag equals `[workspace.package] version` without the leading `v`
2. Runs the [Integration workflow](.github/workflows/integration.yml) as a job: `cargo test --workspace --locked`, which covers the offline tests and the end-to-end suite against a real OpenList server
3. Builds `openlist` for (only when step 1 and 2 succeed):
   - Linux x86_64 and aarch64, both glibc and musl
   - macOS Intel and Apple Silicon
   - Windows x86_64
4. Publishes a GitHub Release with `openlist-<tag>-<target>.tar.gz` / `.zip` archives and a `SHA256SUMS` file
5. Publishes `openlist` and then `openlist-cli` to [crates.io](https://crates.io/crates/openlist), waiting for the library to appear in the index before the CLI (which depends on it) is uploaded

Step 5 needs a crates.io API token with `publish-new` and `publish-update` scopes stored as the repository secret `CARGO_REGISTRY_TOKEN`. Crates already published at that version are skipped, so re-running a release that failed halfway is safe.

Download the latest binaries from the [Releases](https://github.com/Syndim/openlist-rs/releases) page.

```bash
# Bump [workspace.package] version and the openlist version in
# [workspace.dependencies] first, then:
git tag v0.4.0
git push origin v0.4.0
```

Tags containing a hyphen (for example `v0.4.0-rc.1`) are published as pre-releases.

## License

Licensed under the [GNU Affero General Public License v3.0](https://www.gnu.org/licenses/agpl-3.0.html) (AGPL-3.0).

Source code: <https://github.com/Syndim/openlist-rs>
