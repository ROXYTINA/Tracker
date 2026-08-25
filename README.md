# TaskTrack

A production-quality task-management application written in idiomatic Rust, featuring both a CLI and an interactive TUI.

## Features

- **CLI Interface**: Manage tasks directly from your terminal.
- **TUI Interface**: Interactive terminal user interface for a more visual experience.
- **Task Management**: Create, start, complete, reopen, and remove tasks.
- **Priorities & Due Dates**: Set priority (Low, Medium, High) and due dates for your tasks.
- **Filtering & Sorting**: Filter tasks by status and priority, and sort them by creation date, priority, or status.
- **Persistence**: Tasks are saved in a human-readable JSON file.
- **Safe Writing**: Uses a temporary file and rename strategy to prevent data corruption.
- **Colorized Output**: Clear, color-coded terminal output for better readability.

## Requirements

- Rust (Edition 2024)
- Cargo

## Installation & Building

```bash
git clone <repository-url>
cd tasktrack
cargo build --release
```

## CLI Usage

### Add a task
```bash
tasktrack add "Learn Rust ownership" --priority high
```

### List tasks
```bash
tasktrack list
tasktrack list --status todo
tasktrack list --priority high
tasktrack list --sort priority
```

### Manage task status
```bash
tasktrack start 1
tasktrack done 1
tasktrack reopen 1
```

### Remove a task
```bash
tasktrack rm 1
```

### Interactive TUI
```bash
tasktrack tui
```

## TUI Controls

- **Navigate**: `↑`/`↓` or `k`/`j`
- **Toggle Done**: `Enter`
- **Start Task**: `s`
- **Add Task**: `a` (opens input field)
- **Delete Task**: `d`
- **Quit**: `q`
- **Cancel Add**: `Esc`

## Architecture

- **Domain**: Core models (`Task`, `Status`, `Priority`) in `task.rs`.
- **Store**: Persistence logic and task collection management in `store.rs`.
- **Commands**: Application logic layer in `commands.rs`.
- **CLI**: Command-line interface definition using `clap` in `cli.rs`.
- **TUI**: Interactive UI using `ratatui` and `crossterm` in the `tui` module.

## Rust Concepts Demonstrated

- **Ownership & Borrowing**: Efficient use of references and owned types.
- **Enums & Pattern Matching**: Strong typing for task statuses and priorities.
- **Error Handling**: Custom error type using `thiserror` and the `?` operator.
- **Iterators**: Clean filtering and sorting using iterator combinators.
- **Serde**: Serialization and deserialization of task data.
- **Testing**: Comprehensive unit and integration tests.
