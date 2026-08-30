use notify_rust::Notification;

fn main() {
    println!("Sending notification...");
    let mut notification = Notification::new();
    notification
        .summary("Test Notification")
        .body("This is a test notification from tracker.");

    #[cfg(windows)]
    {
        notification.app_id("Windows.SystemToast.Default");
    }

    match notification.show() {
        Ok(_) => println!("Notification sent successfully!"),
        Err(e) => eprintln!("Failed to send notification: {}", e),
    }
}
