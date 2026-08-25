use tasktrack::store::Store;
use tasktrack::task::{Priority, Task, Status};
use tasktrack::tui::app::App;
use chrono::{Utc, Duration};
use tempfile::tempdir;

#[test]
fn test_notification_logic() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("tasks.json");
    let mut store = Store::new(path);
    
    let now = Utc::now();
    
    // Task due in 25 minutes, duration > 59 min
    let mut t1 = Task::new(1, "Task 1".to_string(), Priority::High, Some(now + Duration::minutes(25)));
    t1.created_at = now - Duration::minutes(40); // duration = 65 min
    store.tasks.push(t1);
    
    // Task due in 20 seconds, duration > 1 min
    let mut t2 = Task::new(2, "Task 2".to_string(), Priority::Medium, Some(now + Duration::seconds(20)));
    t2.created_at = now - Duration::minutes(2); // duration = 2m 20s
    store.tasks.push(t2);
    
    // Task already due
    let mut t3 = Task::new(3, "Task 3".to_string(), Priority::Low, Some(now - Duration::minutes(1)));
    t3.created_at = now - Duration::hours(1);
    store.tasks.push(t3);

    let mut app = App::new(store);
    
    // Run tick
    app.tick().unwrap();
    
    let tasks = &app.store.tasks;
    
    // T1 should have notified_30m = true
    assert!(tasks[0].notified_30m, "T1 (30m) should be notified");
    assert!(!tasks[0].notified_30s, "T1 (30s) should NOT be notified yet");
    assert!(!tasks[0].notified_due, "T1 (due) should NOT be notified yet");
    
    // T2 should have notified_30s = true
    assert!(tasks[1].notified_30s, "T2 (30s) should be notified");
    assert!(!tasks[1].notified_due, "T2 (due) should NOT be notified yet");
    
    // T3 should have notified_due = true, and others also true (to avoid re-entry)
    assert!(tasks[2].notified_due, "T3 (due) should be notified");
    assert!(tasks[2].notified_30m, "T3 (30m) should be marked true");
    assert!(tasks[2].notified_30s, "T3 (30s) should be marked true");

    // Test: Completed task should not notify
    let mut t4 = Task::new(4, "Task 4".to_string(), Priority::Low, Some(now - Duration::minutes(1)));
    t4.status = Status::Done;
    app.store.tasks.push(t4);
    app.tick().unwrap();
    assert!(!app.store.tasks[3].notified_due, "Completed task should NOT notify");

    // Test: Changing due date resets flags
    let t3_mut = app.store.get_task_mut(3).unwrap();
    t3_mut.set_due_date(Some(now + Duration::hours(1)));
    assert!(!t3_mut.notified_due);
    assert!(!t3_mut.notified_30m);
    assert!(!t3_mut.notified_30s);
}
