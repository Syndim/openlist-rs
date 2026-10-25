use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub const DEFAULT_SERVER: &str = "http://localhost:5244";

/// Command-line interface for the OpenList API.
#[derive(Debug, Parser)]
#[command(name = "openlist", version, about, long_about = None)]
pub struct Cli {
    /// Base URL of the OpenList server
    #[arg(long, global = true, env = "OPENLIST_SERVER", value_name = "URL")]
    pub server: Option<String>,

    /// API token; overrides any stored credential
    #[arg(long, global = true, env = "OPENLIST_TOKEN", value_name = "TOKEN")]
    pub token: Option<String>,

    /// Print machine-readable JSON instead of human-readable output
    #[arg(long, global = true)]
    pub json: bool,

    /// Print request and response details for each API call (to stderr)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authenticate and store the resulting API token
    Login {
        #[arg(short, long)]
        username: String,

        #[arg(short, long)]
        password: String,

        /// One-time password for accounts with 2FA enabled
        #[arg(long, value_name = "CODE")]
        otp_code: Option<String>,
    },

    /// Remove stored credentials
    Logout,

    /// Show the currently authenticated user
    Me,

    /// List directory contents
    Ls {
        /// Directory path (defaults to /)
        path: Option<String>,

        /// Bypass the server-side cache
        #[arg(long)]
        refresh: bool,

        #[arg(long)]
        page: Option<i64>,

        #[arg(long = "per-page")]
        per_page: Option<i64>,
    },

    /// Show metadata for a file or directory
    Stat { path: String },

    /// Create a directory
    Mkdir { path: String },

    /// Rename an entry in place
    Rename { path: String, new_name: String },

    /// Move entries to another directory
    Mv {
        src_dir: String,
        dst_dir: String,

        #[arg(required = true)]
        names: Vec<String>,
    },

    /// Copy entries to another directory
    Cp {
        src_dir: String,
        dst_dir: String,

        #[arg(required = true)]
        names: Vec<String>,
    },

    /// Remove entries
    Rm {
        dir: String,

        #[arg(required = true)]
        names: Vec<String>,

        /// Do not prompt for confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Search for files and directories
    Search {
        parent: String,
        keywords: String,

        #[arg(long)]
        scope: Option<i32>,

        #[arg(long)]
        page: Option<i64>,

        #[arg(long = "per-page")]
        per_page: Option<i64>,
    },

    /// Upload a local file
    Upload {
        local: PathBuf,
        remote_dir: String,

        /// Remote file name (defaults to the local file name)
        #[arg(long)]
        name: Option<String>,
    },

    /// Download a remote file
    Download {
        remote: String,

        /// Local destination (defaults to the remote file name)
        local: Option<PathBuf>,
    },

    /// Manage share links
    #[command(subcommand)]
    Share(ShareCommand),

    /// Administrative operations
    #[command(subcommand)]
    Admin(AdminCommand),
}

#[derive(Debug, Subcommand)]
pub enum ShareCommand {
    /// List share links
    List {
        #[arg(long)]
        page: Option<i64>,

        #[arg(long = "per-page")]
        per_page: Option<i64>,
    },

    /// Show a share link
    Get { id: String },

    /// Create a share link
    Create {
        #[arg(required = true)]
        paths: Vec<String>,

        #[arg(long)]
        password: Option<String>,

        /// Expiration time (for example "0" for never)
        #[arg(long)]
        expiration: Option<String>,
    },

    /// Update a share link
    Update {
        id: String,

        #[arg(long)]
        path: Option<String>,

        #[arg(long)]
        password: Option<String>,

        #[arg(long)]
        expiration: Option<String>,

        #[arg(long)]
        disable: Option<bool>,
    },

    /// Delete a share link
    Delete { id: String },

    /// Enable a share link
    Enable { id: String },

    /// Disable a share link
    Disable { id: String },
}

#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Manage users
    #[command(subcommand)]
    Users(UsersCommand),

    /// Manage storage mounts
    #[command(subcommand)]
    Storages(StoragesCommand),
}

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    /// List users
    List {
        #[arg(long)]
        page: Option<i64>,

        #[arg(long = "per-page")]
        per_page: Option<i64>,
    },

    /// Show a user
    Get { id: i64 },

    /// Create a user
    Create {
        username: String,

        #[arg(short, long)]
        password: String,

        #[arg(long)]
        base_path: Option<String>,

        #[arg(long)]
        role: Option<i32>,

        #[arg(long)]
        permission: Option<i32>,

        #[arg(long)]
        disabled: bool,
    },

    /// Delete a user
    Delete { id: i64 },
}

#[derive(Debug, Subcommand)]
pub enum StoragesCommand {
    /// List storage mounts
    List {
        #[arg(long)]
        page: Option<i64>,

        #[arg(long = "per-page")]
        per_page: Option<i64>,
    },

    /// Show a storage mount
    Get { id: i64 },

    /// Enable a storage mount
    Enable { id: i64 },

    /// Disable a storage mount
    Disable { id: i64 },

    /// Create a storage mount backed by a local directory
    StorageCreate {
        /// Local directory to expose
        root: PathBuf,

        /// Mount path inside OpenList
        #[arg(long, default_value = "/")]
        mount_path: String,

        /// Cache expiration in minutes
        #[arg(long)]
        cache_expiration: Option<i64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("arguments should parse")
    }

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_global_options_after_subcommand() {
        let cli = parse(&[
            "openlist",
            "ls",
            "/docs",
            "--json",
            "--server",
            "http://example.test",
        ]);
        assert!(cli.json);
        assert_eq!(cli.server.as_deref(), Some("http://example.test"));
        match cli.command {
            Command::Ls { path, .. } => assert_eq!(path.as_deref(), Some("/docs")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_verbose_flag() {
        let cli = parse(&["openlist", "ls", "/", "-v"]);
        assert!(cli.verbose);
    }

    #[test]
    fn parses_nested_admin_subcommand() {
        let cli = parse(&["openlist", "admin", "users", "list", "--per-page", "25"]);
        match cli.command {
            Command::Admin(AdminCommand::Users(UsersCommand::List { page, per_page })) => {
                assert_eq!(page, None);
                assert_eq!(per_page, Some(25));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_share_create() {
        let cli = parse(&[
            "openlist",
            "share",
            "create",
            "/a",
            "/b",
            "--password",
            "secret",
        ]);
        match cli.command {
            Command::Share(ShareCommand::Create {
                paths, password, ..
            }) => {
                assert_eq!(paths, vec!["/a", "/b"]);
                assert_eq!(password.as_deref(), Some("secret"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn requires_at_least_one_remove_target() {
        assert!(Cli::try_parse_from(["openlist", "rm", "/docs"]).is_err());

        let cli = parse(&["openlist", "rm", "/docs", "a.txt"]);
        match cli.command {
            Command::Rm { names, .. } => assert_eq!(names, vec!["a.txt"]),
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
