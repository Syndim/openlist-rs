set shell := ['nu', '-c']
set dotenv-load := true

# OpenList server binary used by `just integration` and `just server`.
# Override with OPENLIST_TEST_SERVER_BIN in the environment (or .envrc).
server_bin := env('OPENLIST_TEST_SERVER_BIN', env('HOME', '') + '/.local/bin/openlist')
# Admin password used by `just server` when it starts an instance.
admin_password := env('OPENLIST_ADMIN_PASSWORD', 'integration-test-password')

[private]
default:
    just -l

# Format, lint and run the offline test suite.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace --locked

# Format the workspace.
fmt:
    cargo fmt --all

# Run Clippy over every target.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run the offline (mockito-backed) test suite.
test:
    cargo test --workspace --locked

# Download, run the full test suite and remove the latest server.
e2e filter="": server-install integration server-remove

# All tests (offline + end-to-end) exactly as CI runs them.
integration filter="": server-install
    #!/usr/bin/env nu
    with-env { OPENLIST_TEST_SERVER_BIN: '{{ server_bin }}' } {
        ^cargo test --workspace --locked -- --nocapture {{ filter }}
    }

# Delete the downloaded OpenList server binary (skipped while `just integration` runs).
server-remove:
    @just _remove-if-idle "{{ server_bin }}"

# Remove the server binary unless the integration tests are running.
# As a dependency of `just e2e` this recipe runs at the same time as
# `just integration`, so it must not delete a binary that is still in use.
# The pattern matches the servers the tests spawn (`openlist --data
# <tmp>/openlist-rs-integration-*/server-data`), not the binary path itself,
# which `pgrep -f` would always match when pointed at this recipe.
[private]
_remove-if-idle path:
    #!/usr/bin/env nu
    let running = ((^pgrep -f openlist-rs-integration- | complete).exit_code == 0)
    if $running {
        print $"integration tests are running, keeping {{ path }}"
        return
    }
    just _remove "{{ path }}"

# Remove a file if it exists.
[private]
_remove path:
    #!/usr/bin/env nu
    let path = "{{ path }}"
    if ($path | path exists) {
        print $"removing ($path)"
        rm -f -- $path
    } else {
        print $"nothing to remove: ($path)"
    }

# Download the latest OpenList server binary into ~/.local/bin.
server-install:
    #!/usr/bin/env nu
    let bin = "{{ server_bin }}"
    if ($bin | path exists) {
        print $"server binary already present: ($bin)"
        return
    }
    let os = if ($nu.os-info.name == "macos") { "darwin" } else { "linux" }
    let arch = if ($nu.os-info.arch | str contains "x86") { "amd64" } else { "arm64" }
    let url = "https://github.com/OpenListTeam/OpenList/releases/latest/download/openlist-" + $os + "-" + $arch + ".tar.gz"
    let tmp = (mktemp -d)
    print $"downloading ($url)"
    ^curl -fsSL --retry 3 -o ($tmp | path join "openlist.tar.gz") $url
    ^tar -xzf ($tmp | path join "openlist.tar.gz") -C $tmp
    mkdir ($bin | path dirname)
    mv -f ($tmp | path join "openlist") $bin
    ^chmod +x $bin
    rm -rf $tmp
    ^$bin version

# Build the CLI in release mode.
build:
    cargo build --release --locked -p openlist-cli

# Run the openlist CLI, e.g. `just cli login -u admin -p admin`.
cli *args:
    @cargo run -q -p openlist-cli -- {{ args }}

# Start a throw-away OpenList server (Ctrl-C to stop) on 127.0.0.1:PORT.
server port="5244" data_dir="":
    #!/usr/bin/env nu
    let dir = if ("{{ data_dir }}" | is-empty) {
        mktemp -d
    } else {
        "{{ data_dir }}" | path expand
    }
    mkdir $dir
    { scheme: { address: "127.0.0.1", http_port: ({{ port }} | into int) } }
        | to json --indent 2
        | save -f ($dir | path join "config.json")
    print $"OpenList data dir: ($dir)"
    print $"Server:            http://127.0.0.1:{{ port }}"
    print "Stop with Ctrl-C."
    with-env { OPENLIST_ADMIN_PASSWORD: '{{ admin_password }}' } {
        ^"{{ server_bin }}" --data $dir server
    }
