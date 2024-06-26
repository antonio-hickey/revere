mod config;
mod error;
mod notification;
mod window;

use config::Config;
use dbus::{blocking::Connection, message::MatchRule};
use error::RevereError;
use notification::Notification;
use std::{
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant},
};
use window::NotificationWindow;

// Hacked up prototype to just set up a connection
// with D-Bus, listen for all messages, and parse
// them into a `Notification` to display with a
// notification window for a few seconds.
//
// TODO:
//     * Fix the issue of youtube notifications showing 
//       without thumbnail first time.
//     * figure out a default UI that looks nice
//     * guess I can support XOrg as well

pub fn main() -> Result<(), RevereError> {
    // Find user config file or use default config
    let config = Config::find();

    // Connect to the DBus session bus
    let bus_cnx = Connection::new_session()?;

    // Wrap the D-Bus connection in Arc for thread safety
    let bus_cnx = Arc::new(bus_cnx);

    // The hash of the last notification which is
    // used for filtering out duplicate D-Bus messages
    let mut last_notification_hash: u64 = 0;

    // Build a rule listening for all messages on the D-Bus
    // TODO: read more of D-Bus docs for how to actually query this.
    let match_rule = MatchRule::new();
    bus_cnx.add_match(match_rule.clone(), move |_: (), _cnx, msg| {
        println!("Received a message: {:?}", msg);
        let notification = Notification::from(msg);
        println!("Parsed into notification: {notification:?}");

        // Hash the notification
        let mut hasher_state = DefaultHasher::new();
        notification.hash(&mut hasher_state);
        let notification_hash = hasher_state.finish();

        // Validate the notification is not a duplicate
        if notification_hash != last_notification_hash {
            // Only display notifications with a title
            if let Some(title) = notification.title {
                let mut thumbnail = notification
                    .image
                    .as_ref()
                    .and_then(|image| File::open(image).ok());

                // Create a new mutable instance of `NotificationWindow`
                let mut notification_window = NotificationWindow::try_new(&config.window)
                    .expect("Failed to crate notification window");

                // Render the notification window for some time duration (default: 3 seconds)
                let start_time = Instant::now();
                while start_time.elapsed() < Duration::from_secs(config.window.duration as u64) {
                    notification_window
                        .event_queue
                        .dispatch(&mut (), |_, _, _| {})
                        .unwrap();
                    notification_window.draw(&title, &mut thumbnail, &config.window);
                }
                notification_window.flush_display().ok();
            }
        }

        // Update the last notification hash to the current one now that
        // this notification is finished (notification window closed)
        last_notification_hash = notification_hash;

        true
    })?;

    // Keep it running forever eva
    loop {
        bus_cnx.process(std::time::Duration::from_millis(1000))?;
    }
}
