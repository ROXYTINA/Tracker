use crate::store::SortMode;
use crate::task::{Priority, Status};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "tasktrack")]
#[command(about = "A simple task manager CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CliStatus {
    Todo,
    InProgress,
    Done,
}

impl From<CliStatus> for Status {
    fn from(s: CliStatus) -> Self {
        match s {
            CliStatus::Todo => Status::Todo,
            CliStatus::InProgress => Status::InProgress,
            CliStatus::Done => Status::Done,
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
