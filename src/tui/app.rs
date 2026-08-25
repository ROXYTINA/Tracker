use ratatui::crossterm::event::{KeyCode, KeyEvent};
use crate::error::AppError;
use crate::store::Store;
use crate::task::{Priority, Status};
use crate::commands;
use crate::notification;
use chrono::{Local, Utc};

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Adding,
    EditingTitle,
    EditingDueDate,
    Help,
}

pub struct App {
    pub store: Store,
    pub selected_index: usize,
    pub should_quit: bool,
    pub input: String,
    pub input_mode: InputMode,
}

impl App {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            selected_index: 0,
            should_quit: false,
            input: String::new(),
            input_mode: InputMode::Normal,
        }
    }

    pub fn tick(&mut self) -> Result<(), AppError> {
        let now = Utc::now();
        let mut changed = false;

        for task in self.store.tasks.iter_mut() {
            if matches!(task.status, Status::Done) {
                continue;
            }

            if let Some(due) = task.due_date {
                let duration = due - task.created_at;

                if now >= due {
                    if !task.notified_due {
                        notification::notify("Task Due", &format!("Task '{}' is now due!", task.title));
                        task.notified_due = true;
                        task.notified_30m = true;
                        task.notified_30s = true;
                        changed = true;
                    }
                } else {
                    // 30 minutes before if duration > 59 minutes
                    if duration > chrono::Duration::minutes(59) && !task.notified_30m {
                        if now >= due - chrono::Duration::minutes(30) {
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
            self.store.save()?;
        }
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key_event),
            InputMode::Adding | InputMode::EditingTitle | InputMode::EditingDueDate => {
                self.handle_input_mode(key_event)
            }
            InputMode::Help => self.handle_help_mode(key_event),
        }
    }

    fn handle_normal_mode(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') => self.input_mode = InputMode::Help,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.store.tasks.is_empty() && self.selected_index < self.store.tasks.len() - 1
                {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(task) = self.store.tasks.get(self.selected_index) {
                    let id = task.id;
                    if matches!(task.status, Status::Done) {
                        commands::reopen_task(&mut self.store, id)?;
                    } else {
                        commands::complete_task(&mut self.store, id)?;
                    }
                }
            }
            KeyCode::Char('s') => {
                if let Some(task) = self.store.tasks.get(self.selected_index) {
                    let id = task.id;
                    commands::start_task(&mut self.store, id)?;
                }
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Adding;
                self.input.clear();
            }
            KeyCode::Char('e') => {
                if let Some(task) = self.store.tasks.get(self.selected_index) {
                    self.input_mode = InputMode::EditingTitle;
                    self.input = task.title.clone();
                }
            }
            KeyCode::Char('t') => {
                if let Some(task) = self.store.tasks.get(self.selected_index) {
                    self.input_mode = InputMode::EditingDueDate;
                    self.input = task
                        .due_date
                        .map(|d| d.with_timezone(&Local).to_rfc3339())
                        .unwrap_or_default();
                }
            }
            KeyCode::Char('d') => {
                if !self.store.tasks.is_empty() {
                    let id = self.store.tasks[self.selected_index].id;
                    commands::remove_task(&mut self.store, id)?;
                    if self.selected_index >= self.store.tasks.len() && self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_input_mode(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Enter => {
                match self.input_mode {
                    InputMode::Adding => {
                        if !self.input.is_empty() {
                            commands::add_task(
                                &mut self.store,
                                self.input.clone(),
                                Priority::Medium,
                                None,
                            )?;
                        }
                    }
                    InputMode::EditingTitle => {
                        if let Some(task) = self.store.tasks.get(self.selected_index) {
                            let id = task.id;
                            commands::edit_task(
                                &mut self.store,
                                id,
                                Some(self.input.clone()),
                                None,
                                None,
                            )?;
                        }
                    }
                    InputMode::EditingDueDate => {
                        if let Some(task) = self.store.tasks.get(self.selected_index) {
                            let id = task.id;
                            let due_date = if self.input.trim().is_empty() {
                                Some(None)
                            } else {
                                crate::task::parse_due_date(&self.input).map(Some)
                            };
                            commands::edit_task(&mut self.store, id, None, None, due_date)?;
                        }
                    }
                    _ => {}
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_mode(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('h') => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }
}
