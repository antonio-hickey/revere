use dbus::arg::{RefArg, Variant};
use dbus::message::MessageType;
use dbus::Message;
use std::collections::HashMap;

/// The Revere Notification type
#[derive(Debug)]
pub struct Notification {
    pub kind: MessageType,
    pub path: String,
    pub title: String,
    pub summary: String,
}
impl From<&Message> for Notification {
    /// Implement DBus Message conversion into a Notification
    fn from(msg: &Message) -> Self {
        let kind = msg.msg_type();
        let path = msg.path().unwrap().to_string();
        let mut title = String::new();
        let mut summary = String::new();

        // Parse out a title/action from a PropertiesChanged member message
        if msg.member().expect("some interface member").to_string()
            == String::from("PropertiesChanged")
        {
            title = {
                // Iterate over the message arguments
                let mut iter = msg.iter_init();
                // Skip the first iteration
                iter.next();

                // Try to parse the title/action
                let title_hashmap: HashMap<String, Variant<String>> = iter.read().unwrap();
                title_hashmap
                    .values()
                    .last()
                    .expect("Some string")
                    .as_str()
                    .expect("Some string")
                    .to_owned()
            };
        }

        Notification {
            kind,
            path,
            title,
            summary: String::new(),
        }
    }
}
