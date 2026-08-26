use ratatui::crossterm::event::{KeyCode, KeyEvent};
use crate::error::AppError;
use crate::store::Store;
use crate::task::{Priority, Status};
use crate::commands;
use chrono::Local;

// Different input modes of the application.
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
        // TUI doesn't handle background notifications anymore
        // It's handled by the daemon process.
        Ok(())
    }

    
    //handle keyboard input event
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key_event),
            InputMode::Adding | InputMode::EditingTitle | InputMode::EditingDueDate => {
                self.handle_input_mode(key_event)
            }
            InputMode::Help => self.handle_help_mode(key_event),
        }
    }

    //functions to handle different input key for char and navigation in TUI (normal key)
    fn handle_normal_mode(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            
            KeyCode::Char('q') => self.should_quit = true,
            
            KeyCode::Char('h') => self.input_mode = InputMode::Help,
            
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            
            KeyCode::Down => {
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

            KeyCode::Char('p') => {
                if let Some(task) = self.store.tasks.get(self.selected_index) {
                    let id = task.id;
                    commands::passed_task(&mut self.store, id)?;
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

    
    //functions to handle different input key for char and navigation in TUI (complecated key)
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
