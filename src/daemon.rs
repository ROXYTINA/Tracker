use crate::store::Store;
use crate::task::Status;
use crate::notification;
use std::{thread, time::Duration};
use chrono::Utc;
use crate::error::AppError;
use tracing::{info, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let logs_dir = parent.join("logs");
            let _ = std::fs::create_dir_all(&logs_dir);
            let file_appender = tracing_appender::rolling::daily(logs_dir, "tracker-daemon.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::registry()
                .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
                .with(fmt::layer().with_writer(non_blocking))
                .init();
            
            Box::leak(Box::new(_guard));
            return;
        }
    }

    // Fallback to stdout if we can't get exe path or create logs dir
    tracing_subscriber::fmt::init();
}

pub fn run_daemon(mut store: Store) -> Result<(), AppError> {
    init_logging();
    info!("Daemon started");
    
    // Initial load
    let _ = store.load();

    loop {
        if let Err(e) = store.load() {
            error!("Failed to load store: {}", e);
            thread::sleep(Duration::from_secs(30));
            continue;
        }
        let now = Utc::now();
        let mut changed = false;

        for task in store.tasks.iter_mut() {
            if matches!(task.status, Status::Done) {
                continue;
            }

            if let Some(due) = task.due_date {
                let duration = due - task.created_at;

                // Mark task as Passed if overdue
                if now >= due && !matches!(task.status, Status::Passed) {
                    info!("Task '{}' is overdue, marking as passed", task.title);
                    task.passed();
                    changed = true;
                }

                if now >= due {
                    if !task.notified_due {
                        info!("Notifying: Task '{}' is due", task.title);
                        notification::notify(
                            "Task Due",
                            &format!("Task '{}' is now due!", task.title),
                        );
                        task.notified_due = true;
                        task.notified_30m = true;
                        task.notified_30s = true;
                        changed = true;
                    }
                } else {
                    // 30 minutes before if duration > 59 minutes
                    if duration > chrono::Duration::minutes(59) && !task.notified_30m {
                        if now >= due - chrono::Duration::minutes(30) {
                            info!("Notifying (30m): Task '{}' is due soon", task.title);
                            notification::notify(
                                "Task Reminder (30m)",
                                &format!("Task '{}' is due in 30 minutes", task.title),
                            );
                            task.notified_30m = true;
                            changed = true;
                        }
                    }

                    // 30 seconds before if duration > 1 minute
                    if duration > chrono::Duration::minutes(1) && !task.notified_30s {
                        if now >= due - chrono::Duration::seconds(30) {
                            info!("Notifying (30s): Task '{}' is due very soon", task.title);
                            notification::notify(
                                "Task Reminder (30s)",
                                &format!("Task '{}' is due in 30 seconds", task.title),
                            );
                            task.notified_30s = true;
                            changed = true;
                        }
                    }
                }
            }
        }

        if changed {
            if let Err(e) = store.save() {
                error!("Failed to save store: {}", e);
            }
        }

        // Sleep for a while before next check to save CPU
        thread::sleep(Duration::from_secs(5));
    }
}
