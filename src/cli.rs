use crate::store::SortMode;
use crate::task::{Priority, Status};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "tracker")]
#[command(about = "A simple task manager CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DaemonAction {
    /// Start the daemon (default)
    Start,
    /// Stop the running background daemon
    Stop,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new task
    Add {
        /// The title of the task
        title: String,
        /// The priority of the task
        #[arg(short, long, default_value = "medium")]
        priority: CliPriority,
        /// Due date (e.g. 2023-12-31T23:59:59Z)
        #[arg(short, long)]
        due: Option<String>,
    },
    /// List all tasks
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<CliStatus>,
        /// Filter by priority
        #[arg(short, long)]
        priority: Option<CliPriority>,
        /// Sort by field
        #[arg(short = 'S', long)]
        sort: Option<CliSortMode>,
    },
    /// Mark a task as done
    Done {
        /// The ID of the task
        id: u32,
    },
    /// Reopen a task
    Reopen {
        /// The ID of the task
        id: u32,
    },
    /// Start a task
    Start {
        /// The ID of the task
        id: u32,
    },
    /// Mark a task as passed
    Passed {
        /// The ID of the task
        id: u32,
    },
    /// Remove a task
    Rm {
        /// The ID of the task
        id: u32,
    },
    /// Edit an existing task
    Edit {
        /// The ID of the task to edit
        id: u32,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New priority
        #[arg(short, long)]
        priority: Option<CliPriority>,
        /// New due date
        #[arg(short, long)]
        due: Option<String>,
        /// Clear the due date
        #[arg(long)]
        clear_due: bool,
    },
    /// Launch the interactive TUI
    Tui,
    /// Launch the interactive shell
    Shell,
    /// Start or stop the background notification monitor
    Daemon {
        #[command(subcommand)]
        action: Option<DaemonAction>,

        /// Run in background (internal use)
        #[arg(long, hide = true)]
        background: bool,
    },
    /// Manage the background service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServiceAction {
    /// Install the service to start automatically on boot
    Install,
    /// Remove the service from the system
    Uninstall,
    /// Start the background service
    Start,
    /// Stop the background service
    Stop,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CliStatus {
    Todo,
    InProgress,
    Done,
    Passed,
}

impl From<CliStatus> for Status {
    fn from(s: CliStatus) -> Self {
        match s {
            CliStatus::Todo => Status::Todo,
            CliStatus::InProgress => Status::InProgress,
            CliStatus::Done => Status::Done,
            CliStatus ::Passed => Status::Passed,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CliPriority {
    Low,
    Medium,
    High,
}

impl From<CliPriority> for Priority {
    fn from(p: CliPriority) -> Self {
        match p {
            CliPriority::Low => Priority::Low,
            CliPriority::Medium => Priority::Medium,
            CliPriority::High => Priority::High,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CliSortMode {
    Created,
    Priority,
    Status,
}

impl From<CliSortMode> for SortMode {
    fn from(s: CliSortMode) -> Self {
        match s {
            CliSortMode::Created => SortMode::Created,
            CliSortMode::Priority => SortMode::Priority,
            CliSortMode::Status => SortMode::Status,
        }
    }
}
