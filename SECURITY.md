# Security Policy

## Supported versions

Security fixes are released for the latest `0.x` minor line. Older versions are
not patched; upgrade to the newest release before reporting.

| Version | Supported |
|---|---|
| 0.4.x | yes |
| < 0.4 | no |

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Use GitHub's private vulnerability reporting:

<https://github.com/Syndim/openlist-rs/security/advisories/new>

Please include the affected crate and version (`openlist` or `openlist-cli`),
the platform, reproduction steps, and the impact you believe it has. If the
report is about a leaked credential, revoke it first — this project cannot do
that for you.

You can expect an initial acknowledgement within about a week. This is a
volunteer-maintained project, so there is no bug bounty and no guaranteed fix
deadline; confirmed issues are fixed in a release and credited in the advisory
unless you ask to stay anonymous.

## Scope

In scope for this repository:

- The `openlist` SDK crate: request construction, response parsing, error
  handling, TLS and redirect behaviour, credential handling.
- The `openlist-cli` binary: argument parsing, local credential cache (written
  with `0600` permissions on Unix), and anything that writes to disk.
- The release automation in `.github/workflows/`: what gets built from which
  commit and what is published to crates.io.

Out of scope:

- Vulnerabilities in the [OpenList](https://github.com/OpenListTeam/OpenList)
  server itself — report those upstream.
- Misconfiguration of your own server, storage backend, or reverse proxy.
- Findings that require an already-compromised machine or a malicious token.

Dependency vulnerabilities are tracked through Dependabot alerts; if you spot
one that is not flagged, an issue with the advisory link is welcome.
