mod config;
mod error;
mod notification;
mod window;

use config::Config;
use dbus::{
    blocking::Connection, channel::Sender, message::MatchRule, strings::ErrorName, Message, Path,
};
use error::Error;
use notification::Notification;
use std::{
    ffi::CString,
    fs::File,
    time::{Duration, Instant},
};
use window::NotificationWindow;

// Hacked up prototype to just set up a connection
// with D-Bus, listen for messages on the notifications
// interface, and parse them into a `Notification` to
// display with a notification window for a few seconds.
//
// TODO:
//     * Fix the issue of youtube notifications showing
//       without thumbnail first time.
//     * figure out a default UI that looks nice
//     * guess I can support XOrg as well

/// The default notification icon for Revere
static DEFAULT_ICON_PNG: &[u8] = include_bytes!("../assets/notification-icon.png");

pub fn main() -> Result<(), Error> {
    // Find user config file or use default config
    let config = Config::find();

    // Connect to the DBus session bus and claim the
    // name space for the system's notification daemon
    let bus_cnx = Connection::new_session()?;
    bus_cnx.request_name("org.freedesktop.Notifications", false, true, false)?;

    // Build a rule registering to the D-Bus notifications
    // interface and listening for `Notify` method calls.
    let mut match_rule = MatchRule::new_method_call();
    match_rule.interface = Some("org.freedesktop.Notifications".into());
    match_rule.path =
        Some(Path::new("/org/freedesktop/Notifications").expect("Valid D-Bus Object Path"));

    bus_cnx.add_match(match_rule.clone(), move |_: (), cnx, msg| {
        let member_name = msg.member().map(|m| m.to_string());

        // Handle the different Notification interface methods
        let reply = match member_name.as_deref() {
            Some("Notify") => handle_notify(msg, &config),
            Some("GetCapabilities") => handle_get_capabilities(msg),
            Some("GetServerInformation") => handle_get_server_information(msg),
            _ => {
                eprintln!("Unknown method called: {:?}", msg.member());
                Message::error(
                    msg,
                    &ErrorName::from_slice("org.freedesktop.DBus.Error.UnknownMethod").unwrap(),
                    &CString::new("Unknown method").expect("CString::new failed"),
                )
            }
        };

        // Respond with true or false based
        // on the result of sending our reply
        cnx.send(reply).is_ok()
    })?;

    // Keep it running forever eva
    loop {
        bus_cnx.process(std::time::Duration::from_millis(1000))?;
    }
}

/// Handle the Notify method for the Notifications Interface
fn handle_notify(msg: &Message, config: &Config) -> Message {
    // Parse the message into a `Notification`
    let notification = Notification::from(msg);

    // Extract the icon image from the notification
    let icon = notification
        .icon
        .as_ref()
        .and_then(|image| File::open(image).ok())
        .map(|f| Box::new(f) as Box<dyn std::io::Read>)
        .unwrap_or_else(|| {
            // No thumbnail or notification icon provided
            // so use the default revere notification icon
            Box::new(std::io::Cursor::new(DEFAULT_ICON_PNG)) as Box<dyn std::io::Read>
        });

    // Create an Image Surface for the notification icon
    let image_surface =
        NotificationWindow::create_image_surface(icon).expect("Failed to create image surface");

    // Create a new mutable instance of `NotificationWindow`
    let mut notification_window =
        NotificationWindow::try_new(&config.window).expect("Failed to crate notification window");

    // Render the notification window for some time duration (default: 3 seconds)
    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(config.window.duration as u64) {
        // Try to handle the dispatching of events on the notification window
        if let Err(e) = notification_window
            .event_queue
            .dispatch(&mut (), |_, _, _| {})
        {
            eprintln!("Failed to dispatch event: {e:?}");
        }

        // Try to render the notification window
        if let Err(e) = notification_window.draw(
            &notification.summary.clone().unwrap_or_default(),
            &notification.body.clone().unwrap_or_default(),
            &image_surface,
            &config.window,
        ) {
            eprintln!("Error drawing notification window: {e:?}");
        }
    }

    // Flush the display of the notification window
    notification_window.flush_display().ok();

    // Respond to the client with 1 indicating success
    Message::method_return(msg).append1(1u32)
}

/// Handle the GetCapabilities method for the Notifications Interface
fn handle_get_capabilities(msg: &Message) -> Message {
    let capabilities = vec!["actions".to_string(), "body".to_string()];
    Message::method_return(msg).append1(capabilities)
}

/// Handle the GetServerInformation method for the Notifications Interface
fn handle_get_server_information(msg: &Message) -> Message {
    Message::method_return(msg)
        // Service Metadata
        .append3(
            String::from("Revere"),
            String::from("ByteForge"),
            String::from("1.0"),
        )
        // The supported spec version
        .append1(String::from("1.2"))
}
