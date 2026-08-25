use clap::Parser;
use std::path::PathBuf;
use tasktrack::cli::{Cli, Commands};
use tasktrack::commands;
use tasktrack::error::AppError;
use tasktrack::store::Store;
use tasktrack::tui;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    let path = std::env::var("TASK_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            p.push("tasks.json");
            p
        });

    let mut store = Store::new(path);
    store.load()?;

    match cli.command {
        Commands::Add {
            title,
            priority,
            due,
        } => {
            let due_date = due.and_then(|d| tasktrack::task::parse_due_date(&d));
            match commands::add_task(&mut store, title, priority.into(), due_date) {
                Ok(id) => println!("Added task with ID: {}", id),
                Err(e) => eprintln!("{}", e),
            }
        }
        Commands::List {
            status,
            priority,
            sort,
        } => {
            let tasks = commands::list_tasks(
                &store,
                status.map(Into::into),
                priority.map(Into::into),
                sort.map(Into::into),
            );
            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in tasks {
                    println!("{}", task);
                }
            }
        }
        Commands::Done { id } => match commands::complete_task(&mut store, id) {
            Ok(_) => println!("Task {} marked as done.", id),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Commands::Reopen { id } => match commands::reopen_task(&mut store, id) {
            Ok(_) => println!("Task {} reopened.", id),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Commands::Start { id } => match commands::start_task(&mut store, id) {
            Ok(_) => println!("Task {} started.", id),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Commands::Rm { id } => match commands::remove_task(&mut store, id) {
            Ok(_) => println!("Task {} removed.", id),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Commands::Edit {
            id,
            title,
            priority,
            due,
            clear_due,
        } => {
            let due_date = if clear_due {
                Some(None)
            } else {
                due.and_then(|d| tasktrack::task::parse_due_date(&d)).map(Some)
            };

            match commands::edit_task(
                &mut store,
                id,
                title,
                priority.map(Into::into),
                due_date,
            ) {
                Ok(_) => println!("Task {} updated.", id),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Tui => {
            tui::run(store)?;
        }
    }

    Ok(())
}
