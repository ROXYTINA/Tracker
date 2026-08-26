use notify_rust::Notification;

pub fn notify(title: &str, message: &str) {
    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(message);

    #[cfg(windows)]
    {
        // On Windows, notifications need an App ID. If we're not installed, 
        // we use a generic one or none, but notify-rust prefers one.
        // Let's use a more standard one or try without it if it fails.
        notification.app_id("Windows.SystemToast.Default");
    }

    let _ = notification.show();
}
