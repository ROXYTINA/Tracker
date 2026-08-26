use chrono::{DateTime, Utc, Local};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Passed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn level(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notified_due: bool,
    #[serde(default)]
    pub notified_30m: bool,
    #[serde(default)]
    pub notified_30s: bool,
}

impl Task {
    pub fn new(
        id: u32,
        title: String,
        priority: Priority,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            title,
            status: Status::Todo,
            priority,
            created_at: Utc::now(),
            due_date,
            notified_due: false,
            notified_30m: false,
            notified_30s: false,
        }
    }

    pub fn complete(&mut self) {
        self.status = Status::Done;
    }

    pub fn reopen(&mut self) {
        self.status = Status::Todo;
    }

    pub fn start(&mut self) {
        self.status = Status::InProgress;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    pub fn passed(&mut self) {
        self.status = Status::Passed;
    }

    pub fn set_due_date(&mut self, due_date: Option<DateTime<Utc>>) {
        self.due_date = due_date;
        self.notified_due = false;
        self.notified_30m = false;
        self.notified_30s = false;
    }
}

pub fn parse_due_date(input: &str) -> Option<DateTime<Utc>> {
    if input.starts_with('+') {
        let duration_str = &input[1..];
        let mut number_str = String::new();
        let mut unit_str = String::new();
        let mut parsing_number = true;

        for c in duration_str.chars() {
            if parsing_number && c.is_ascii_digit() {
                number_str.push(c);
            } else if !c.is_whitespace() {
                parsing_number = false;
                unit_str.push(c);
            }
        }

        let count: i64 = number_str.parse().ok()?;
        let now = Utc::now();

        match unit_str.as_str() {
            "mn" | "min" => Some(now + chrono::Duration::minutes(count)),
            "h" => Some(now + chrono::Duration::hours(count)),
            "d" => Some(now + chrono::Duration::days(count)),
            "w" => Some(now + chrono::Duration::weeks(count)),
            _ => None,
        }
    } else {
        DateTime::parse_from_rfc3339(input)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let status_str = if let Some(due) = self.due_date {
            if Utc::now() >= due && !matches!(self.status, Status::Done) && !matches!(self.status, Status::Passed) {
                "OVERDUE".red()
            } else {
                match self.status {
                    Status::Todo => "TODO".white(),
                    Status::InProgress => "IN PROGRESS".yellow(),
                    Status::Done => "DONE".green(),
                    Status::Passed => "PASSED".blue(),
                }
            }
        } else {
            match self.status {
                Status::Todo => "TODO".white(),
                Status::InProgress => "IN PROGRESS".yellow(),
                Status::Done => "DONE".green(),
                Status::Passed => "PASSED".blue(),
            }
        };

        let priority_str = match self.priority {
            Priority::Low => "LOW".blue(),
            Priority::Medium => "MEDIUM".yellow(),
            Priority::High => "HIGH".red(),
        };

        write!(
            f,
            "[{}] {} ({}) [Priority: {}]",
            self.id, self.title, status_str, priority_str
        )?;

        if let Some(due) = self.due_date {
            let local_due = due.with_timezone(&Local);
            write!(f, " [Due: {}]", local_due.format("%Y-%m-%d %H:%M"))?;
        }

        Ok(())
    }
}

