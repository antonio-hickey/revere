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
use std::ffi::CString;

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
            Some("Notify") => handle_notify(msg, config),
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

        // Reply with an indication that we
        // understood the message or not.
        cnx.send(reply).is_ok()
    })?;

    // Keep it running forever eva
    loop {
        bus_cnx.process(std::time::Duration::from_millis(1000))?;
    }
}

/// Handle the Notify method for the Notifications Interface
fn handle_notify(msg: &Message, config: Config) -> Message {
    // Spawn a new thread to parse and display the notification
    std::thread::spawn({
        let notification = Notification::from(msg);
        move || notification.display_in_new_window(&config)
    });

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
