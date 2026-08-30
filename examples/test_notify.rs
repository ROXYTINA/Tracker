use notify_rust::Notification;

fn main() {
    println!("Sending notification...");
    let result = Notification::new()
        .app_id("com.kanha.tracker")
        .summary("Notification Test")
        .body("If you see this, notifications are working!")
        .show();

    match result {
        Ok(_) => println!("Notification sent successfully!"),
        Err(e) => println!("Error sending notification: {:?}", e),
    }
}
