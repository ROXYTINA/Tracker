use crate::task::{Priority, Status};
use crate::tui::app::{App, InputMode};
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title / Header
    let title = Paragraph::new(format!(
        "TaskTrack - Total Tasks: {}",
        app.store.tasks.len()
    ))
    .block(Block::default().borders(Borders::ALL).title("Info"));
    f.render_widget(title, chunks[0]);

    // Task List
    let items: Vec<ListItem> = app
        .store
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_symbol = match task.status {
                Status::Todo => "○",
                Status::InProgress => "→",
                Status::Done => "✓",
            };

            let status_color = match task.status {
                Status::Todo => Color::White,
                Status::InProgress => Color::Yellow,
                Status::Done => Color::Green,
            };

            let priority_color = match task.priority {
                Priority::Low => Color::Blue,
                Priority::Medium => Color::Yellow,
                Priority::High => Color::Red,
            };

            let mut spans = vec![
                Span::styled(
                    format!("{} ", status_symbol),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("[{}] ", task.id),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(task.title.clone(), style),
                Span::raw(" ("),
                Span::styled(
                    format!("{:?}", task.status).to_uppercase(),
                    Style::default().fg(status_color),
                ),
                Span::raw(") [P: "),
                Span::styled(
                    format!("{:?}", task.priority).to_uppercase(),
                    Style::default().fg(priority_color),
                ),
                Span::raw("]"),
            ];

            if let Some(due) = task.due_date {
                let local_due = due.with_timezone(&Local);
                spans.push(Span::styled(
                    format!(" [Due: {}]", local_due.format("%Y-%m-%d %H:%M")),
                    Style::default().fg(Color::Magenta),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, chunks[1], &mut state);

    // Footer / Controls
    let footer_text = match app.input_mode {
        InputMode::Normal => {
            "↑↓/kj: Nav | Enter: Done | s: Start | a: Add | e: Edit | t: Time | d: Del | h: Help | q: Quit"
        }
        InputMode::Adding => "Adding Task: Enter to Save | Esc to Cancel",
        InputMode::EditingTitle => "Editing Title: Enter to Save | Esc to Cancel",
        InputMode::EditingDueDate => "Setting Due Date (+2h, +1d, ISO8601): Enter to Save | Esc to Cancel",
        InputMode::Help => "Press any key to close help",
    };
    let footer =
        Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, chunks[2]);

    // Input Pop-up
    match app.input_mode {
        InputMode::Adding | InputMode::EditingTitle | InputMode::EditingDueDate => {
            let area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, area);
            let title = match app.input_mode {
                InputMode::Adding => "Add Task Title",
                InputMode::EditingTitle => "Edit Task Title",
                InputMode::EditingDueDate => "Set Due Date (e.g. +2h, +1d)",
                _ => "",
            };
            let input = Paragraph::new(app.input.clone()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title),
            );
            f.render_widget(input, area);
        }
        InputMode::Help => {
            let area = centered_rect(60, 60, f.area());
            f.render_widget(Clear, area);
            let help_text = vec![
                Line::from("Keyboard Shortcuts:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("q      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Quit"),
                ]),
                Line::from(vec![
                    Span::styled("h      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Show this help"),
                ]),
                Line::from(vec![
                    Span::styled("↑↓/k j ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Navigate tasks"),
                ]),
                Line::from(vec![
                    Span::styled("Enter  ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Toggle task Done/Todo"),
                ]),
                Line::from(vec![
                    Span::styled("s      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Start task (In Progress)"),
                ]),
                Line::from(vec![
                    Span::styled("a      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Add new task"),
                ]),
                Line::from(vec![
                    Span::styled("e      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Edit task title"),
                ]),
                Line::from(vec![
                    Span::styled("t      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Set task due date/duration"),
                ]),
                Line::from(vec![
                    Span::styled("d      ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Delete selected task"),
                ]),
                Line::from(""),
                Line::from("Input Modes:"),
                Line::from(vec![
                    Span::styled("Enter  ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Save changes"),
                ]),
                Line::from(vec![
                    Span::styled("Esc    ", Style::default().fg(Color::Yellow)),
                    Span::raw(": Cancel changes"),
                ]),
                Line::from(""),
                Line::from("Duration format:"),
                Line::from("  +10mn - 10 minutes from now"),
                Line::from("  +5h  - 5 hours from now"),
                Line::from("  +2d  - 2 days from now"),
                Line::from("  +1w  - 1 week from now"),
                Line::from("  +1m  - 1 month from now"),
            ];
            let help = Paragraph::new(help_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help"),
            );
            f.render_widget(help, area);
        }
        _ => {}
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
