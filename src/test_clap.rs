use std::io::{self};
use std::process::Command;

fn main() {

    loop {

        // Read user input
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        // Remove whitespace and newline
        let input = input.trim();

        // Ignore empty input
        if input.is_empty() {
            continue;
        }

        // Special command: notepad
        if input == "notepad" {
            Command::new("notepad.exe")
                .spawn()
                .expect("Failed to start Notepad");

            continue;
        }

        // Try to execute anything else as a Windows command
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