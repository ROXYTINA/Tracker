use notify_rust::Notification;

pub fn notify(title: &str, message: &str) {
    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(message);

    #[cfg(windows)]
    {
        // On Windows, notifications need an App ID.
        // We use a consistent one that matches the installer's registration.
        notification.app_id("com.kanha.tracker");
    }

    let _ = notification.show();
}
