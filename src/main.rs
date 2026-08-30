use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tracker::cli::{Cli, Commands, ServiceAction};
use tracker::commands;
use tracker::error::AppError;
use tracker::store::Store;
use tracker::tui;
use tracker::daemon;
use tracker::service;

#[cfg(windows)]
fn run_service() -> Result<(), AppError> {
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };
    use std::sync::mpsc;
    use std::time::Duration;

    define_windows_service!(ffi_service_main, windows_service_main);

    fn windows_service_main(_arguments: Vec<std::ffi::OsString>) {
        if let Err(_e) = run_windows_service() {
            // Handle error (e.g., log it)
        }
    }

    fn run_windows_service() -> Result<(), windows_service::Error> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register("tracker", event_handler)?;

        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Start our daemon logic in a separate thread or just run it here if it's non-blocking
        // Since run_daemon is a loop, we should probably run it here but check for shutdown
        
        let path = std::env::var("TASK_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                p.push("tasks.json");
                p
            });
        let store = Store::new(path);
        
        // We need a modified run_daemon that can be interrupted
        // For now, let's just spawn it and wait for shutdown
        std::thread::spawn(move || {
            let _ = daemon::run_daemon(store);
        });

        loop {
            if let Ok(_) = shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                break;
            }
        }

        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    service_dispatcher::start("tracker", ffi_service_main)
        .map_err(|e| {
            tracing::error!("Failed to start service dispatcher: {}", e);
            AppError::Other(format!("Failed to start service dispatcher: {}", e))
        })?;
    
    Ok(())
}

fn start_background_daemon() {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("tracker.exe"));
    let _ = Command::new(exe)
        .arg("daemon")
        .arg("--background")
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
        .spawn();
}

fn main() -> Result<(), AppError> {
    // Check if we are running as a Windows service
    #[cfg(windows)]
    {
        // windows-service crate suggests checking if we are running as a service
        // A simple way is to see if we were started with "daemon" and if dispatcher fails when run manually.
        // But more robust is to try to start dispatcher if no args or "daemon" arg.
    }

    let cli = Cli::parse();

    let path = std::env::var("TASK_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    return parent.join("tasks.json");
                }
            }
            let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            p.push("tasks.json");
            p
        });

    let mut store = Store::new(path);
    store.load()?;

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Add {
            title,
            priority,
            due,
        } => {
            let due_date = due.and_then(|d| tracker::task::parse_due_date(&d));
            match commands::add_task(&mut store, title, priority.into(), due_date) {
                Ok(id) => {
                    println!("Added task with ID: {}", id);
                    // Try to ensure daemon is running
                    #[cfg(windows)]
                    {
                        if let Err(_) = service::start_service() {
                            start_background_daemon();
                        }
                    }
                },
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
        Commands::Passed { id } => match commands::passed_task(&mut store, id) {
            Ok(_) => println!("Task {} marked as passed.", id),
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
                due.and_then(|d| tracker::task::parse_due_date(&d)).map(Some)
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
            // Try to ensure daemon is running
            #[cfg(windows)]
            {
                if let Err(_) = service::start_service() {
                    // If service fails to start (e.g. not installed or access denied),
                    // start as background process
                    start_background_daemon();
                }
            }
            
            tui::run(store)?;
        }
        Commands::Daemon { action, background } => {
            if let Some(tracker::cli::DaemonAction::Stop) = action {
                #[cfg(windows)]
                {
                    use sysinfo::System;
                    let s = System::new_all();
                    let current_pid = sysinfo::get_current_pid().unwrap();
                    let mut found = false;
                    for (pid, process) in s.processes() {
                        if process.name() == "tracker.exe" && *pid != current_pid {
                            process.kill();
                            println!("Stopped background daemon (PID: {}).", pid);
                            found = true;
                        }
                    }
                    if !found {
                        println!("No background daemon found running.");
                    }
                }
                #[cfg(not(windows))]
                {
                    println!("Daemon stop not implemented for this platform.");
                }
                return Ok(());
            }

            #[cfg(windows)]
            {
                // Try running as service first
                if let Err(_) = run_service() {
                    // If it fails (e.g. not running as a service), check if we should daemonize
                    if background {
                        daemon::run_daemon(store)?;
                    } else {
                        // Re-run ourselves in the background
                        start_background_daemon();
                        println!("Daemon started in background.");
                    }
                }
            }
            #[cfg(not(windows))]
            {
                daemon::run_daemon(store)?;
            }
        }
        Commands::Service { action } => match action {
            ServiceAction::Install => {
                service::install_service()?;
                println!("Service installed successfully.");
            }
            ServiceAction::Uninstall => {
                service::uninstall_service()?;
                println!("Service uninstalled successfully.");
            }
            ServiceAction::Start => {
                service::start_service()?;
                println!("Service started successfully.");
            }
            ServiceAction::Stop => {
                service::stop_service()?;
                println!("Service stopped successfully.");
            }
        },
        Commands::Status => {
            #[cfg(windows)]
            {
                use sysinfo::System;
                let s = System::new_all();
                let current_pid = sysinfo::get_current_pid().unwrap();
                let mut found = false;
                
                // Check for background process
                for (pid, process) in s.processes() {
                    if process.name() == "tracker.exe" && *pid != current_pid {
                        println!("Background daemon is running (PID: {}).", pid);
                        found = true;
                    }
                }

                if !found {
                    println!("No background daemon is running.");
                }
            }
            #[cfg(not(windows))]
            {
                println!("Status command not fully implemented for this platform.");
            }
        }
        Commands::Shell => {
            loop {
                print!("> ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");

                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                if input == "exit" || input == "quit" {
                    break;
                }

                if input == "tracker" {
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        const DETACHED_PROCESS: u32 = 0x00000008;
                        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
                        const CREATE_NO_WINDOW: u32 = 0x08000000;

                        Command::new("tracker.exe")
                            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
                            .spawn()
                            .expect("Failed to start tracker");
                    }
                    #[cfg(not(windows))]
                    {
                        Command::new("tracker")
                            .spawn()
                            .expect("Failed to start tracker");
                    }
                    continue;
                }

                let status = Command::new("cmd")
                    .args(["/C", input])
                    .status();

                match status {
                    Ok(status) => {
                        if !status.success() {
                            println!("Command failed.");
                        }
                    }
                    Err(error) => {
                        println!("Failed to execute command: {}", error);
                    }
                }
            }
        }
    }

    Ok(())
}
