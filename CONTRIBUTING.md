# Contributing

Thanks for taking the time to help. This document explains how the repository is
laid out, how to build and test it, and how a change gets merged and released.

## Issues and pull requests

- **Bug reports and feature requests:** use
  [GitHub issues](https://github.com/Syndim/openlist-rs/issues).
- **Pull requests:** open them against `master`. Review, CI and merging all
  happen in the pull request. The `Integration` workflow runs the end-to-end
  suite and is a required check, so a change that breaks the build cannot be
  merged.
- **Releases:** a maintainer tags `v<version>` on `master` after
  `just bump-version`; the tag runs the full test suite and publishes the
  binaries and the crates.

## Development setup

- Rust **1.88** or newer (edition 2024, and the code uses let-chains).
- [`just`](https://github.com/casey/just) for the task recipes (optional but
  recommended).
- No database or services are needed for the unit tests; they use
  [`mockito`](https://docs.rs/mockito) and run offline.

```bash
git clone https://github.com/Syndim/openlist-rs.git
cd openlist-rs
just check        # cargo fmt --check + cargo clippy -D warnings
just test         # unit tests
```

`just integration` installs a real OpenList server into a temp directory and runs
the end-to-end suite against it; `just e2e` does the same and also covers the
CLI flows. The `Integration` workflow runs that suite on every push and PR.

## Before you open a pull request

1. `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
   must be clean — CI enforces both.
2. Tests must pass on the MSRV (1.88) as well as stable.
3. New public API needs doc comments; the crate is published to crates.io and
   rustdoc is part of the deliverable.
4. Keep commits focused and write them in
   [Conventional Commits](https://www.conventionalcommits.org/) style
   (`feat(cli): …`, `fix(fs): …`, `docs: …`) — the release notes and the
   changelog are derived from them.
5. Describe the change and its motivation in the PR body. Say what you tested
   and on which platform.

## Adding a command

The CLI is a thin layer over the SDK. The usual shape of a change is:

1. Add or extend the typed request/response models in `crates/openlist/src/models`.
2. Add the client method to the relevant `crates/openlist/src/api` module, with
   a mockito-backed test.
3. Wire the subcommand in `crates/openlist-cli/src`, preferring the shared table
   and JSON output helpers.
4. Extend the integration suite if the endpoint mutates server state.

## Licence

Contributions are accepted under the [AGPL-3.0](LICENSE). There is no CLA and no
copyright assignment; by opening a pull request you agree to license your work
under those terms.

## Security

Please do not report vulnerabilities through issues or pull requests — see
[SECURITY.md](SECURITY.md).
