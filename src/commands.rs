use crate::error::AppError;
use crate::store::{SortMode, Store};
use crate::task::{Priority, Status, Task};
use crate::notification;
use chrono::{DateTime, Utc};

pub fn add_task(
    store: &mut Store,
    title: String,
    priority: Priority,
    due_date: Option<DateTime<Utc>>,
) -> Result<u32, AppError> {
    let task_title = title.clone();
    let task = store.add_task(title, priority, due_date);
    let id = task.id;
    store.save()?;
    notification::notify("Task Added", &format!("Task: {}", task_title));
    Ok(id)
}

pub fn list_tasks(
    store: &Store,
    status: Option<Status>,
    priority: Option<Priority>,
    sort: Option<SortMode>,
) -> Vec<&Task> {
    store.list_tasks(status, priority, sort)
}

pub fn complete_task(store: &mut Store, id: u32) -> Result<(), AppError> {
    let task = store.get_task_mut(id).ok_or(AppError::NotFound(id))?;
    task.complete();
    let title = task.title.clone();
    store.save()?;
    notification::notify("Task Completed", &format!("Task: {}", title));
    Ok(())
}

pub fn reopen_task(store: &mut Store, id: u32) -> Result<(), AppError> {
    let task = store.get_task_mut(id).ok_or(AppError::NotFound(id))?;
    task.reopen();
    let title = task.title.clone();
    store.save()?;
    notification::notify("Task Reopened", &format!("Task: {}", title));
    Ok(())
}

pub fn start_task(store: &mut Store, id: u32) -> Result<(), AppError> {
    let task = store.get_task_mut(id).ok_or(AppError::NotFound(id))?;
    task.start();
    let title = task.title.clone();
    store.save()?;
    notification::notify("Task Started", &format!("Task: {}", title));
    Ok(())
}

pub fn remove_task(store: &mut Store, id: u32) -> Result<(), AppError> {
    let task = store.get_task(id).ok_or(AppError::NotFound(id))?;
    let title = task.title.clone();
    store.remove_task(id)?;
    store.save()?;
    notification::notify("Task Removed", &format!("Task: {}", title));
    Ok(())
}

pub fn edit_task(
    store: &mut Store,
    id: u32,
    title: Option<String>,
    priority: Option<Priority>,
    due_date: Option<Option<DateTime<Utc>>>,
) -> Result<(), AppError> {
    let task = store.get_task_mut(id).ok_or(AppError::NotFound(id))?;

    if let Some(t) = title {
        task.set_title(t);
    }
    if let Some(p) = priority {
        task.set_priority(p);
    }
    if let Some(d) = due_date {
        task.set_due_date(d);
    }

    let task_title = task.title.clone();
    store.save()?;
    notification::notify("Task Edited", &format!("Task: {}", task_title));
    Ok(())
}
