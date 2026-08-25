use notify_rust::Notification;

pub fn notify(title: &str, message: &str) {
    let _ = Notification::new()
        .summary(title)
        .body(message)
        .show();
}
