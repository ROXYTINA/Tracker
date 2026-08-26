use crate::error::AppError;
use crate::task::{Priority, Status, Task};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Store {
    pub tasks: Vec<Task>,
    #[serde(skip)]
    path: PathBuf,
}

impl Store {
    pub fn new(path: PathBuf) -> Self {
        Self {
            tasks: Vec::new(),
            path,
        }
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        if !self.path.exists() {
            self.tasks = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)?;
        self.tasks = serde_json::from_str(&content)?;
        Ok(())
    }


    pub fn save(&self) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(&self.tasks)?;

        // Safe write: temp file then rename
        let mut temp_path = self.path.clone();
        temp_path.set_extension("tmp");

        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, &self.path)?;

        Ok(())
    }

    pub fn add_task(
        &mut self,
        title: String,
        priority: Priority,
        due_date: Option<DateTime<Utc>>,
    ) -> &Task {
        let id = self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let task = Task::new(id, title, priority, due_date);
        self.tasks.push(task);
        self.tasks.last().unwrap()
    }

    pub fn get_task(&self, id: u32) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn get_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn remove_task(&mut self, id: u32) -> Result<(), AppError> {
        let pos = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(AppError::NotFound(id))?;
        self.tasks.remove(pos);
        Ok(())
    }

    pub fn list_tasks(
        &self,
        status_filter: Option<Status>,
        priority_filter: Option<Priority>,
        sort_by: Option<SortMode>,
    ) -> Vec<&Task> {
        let mut filtered: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| status_filter.is_none_or(|s| t.status == s))
            .filter(|t| priority_filter.is_none_or(|p| t.priority == p))
            .collect();

        if let Some(mode) = sort_by {
            match mode {
                SortMode::Created => filtered.sort_by_key(|t| t.created_at),
                SortMode::Priority => {
                    filtered.sort_by(|a, b| b.priority.level().cmp(&a.priority.level()))
                }
                SortMode::Status => filtered.sort_by_key(|t| t.status as u8),
            }
        }

        filtered
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Created,
    Priority,
    Status,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::NamedTempFile;
//
//     #[test]
//     fn test_add_get_remove() {
//         let temp = NamedTempFile::new().unwrap();
//         let mut store = Store::new(temp.path().to_path_buf());
//
//         store.add_task("Task 1".to_string(), Priority::High, None);
//         store.add_task("Task 2".to_string(), Priority::Low, None);
//
//         assert_eq!(store.tasks.len(), 2);
//         assert_eq!(store.get_task(1).unwrap().title, "Task 1");
//
//         store.remove_task(1).unwrap();
//         assert_eq!(store.tasks.len(), 1);
//         assert!(store.get_task(1).is_none());
//     }
//
//     #[test]
//     fn test_persistence() {
//         let temp = NamedTempFile::new().unwrap();
//         let path = temp.path().to_path_buf();
//
//         {
//             let mut store = Store::new(path.clone());
//             store.add_task("Task 1".to_string(), Priority::High, None);
//             store.save().unwrap();
//         }
//
//         {
//             let mut store = Store::new(path);
//             store.load().unwrap();
//             assert_eq!(store.tasks.len(), 1);
//             assert_eq!(store.tasks[0].title, "Task 1");
//         }
//     }
// }
