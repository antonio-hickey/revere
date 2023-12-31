use dbus::arg::{self, PropMap, RefArg, Variant};
use dbus::message::MessageType;
use dbus::Message;
use std::collections::HashMap;

/// The Revere Notification type
#[derive(Debug)]
pub struct Notification {
    pub kind: MessageType,
    pub path: String,
    pub title: Option<String>,
    pub summary: Option<String>,
}
impl From<&Message> for Notification {
    /// Implement DBus Message conversion into a Notification
    fn from(msg: &Message) -> Self {
        let kind = msg.msg_type();
        let path = msg.path().unwrap().to_string();
        let mut title = None;
        let mut summary = None;

        // Parse out a title/action from a PropertiesChanged member message
        if msg.member().expect("some interface member").to_string() == *"PropertiesChanged" {
            title = {
                // Iterate over the message arguments
                let mut iter = msg.iter_init();
                // Skip the first iteration
                iter.next();

                // Parse the D-BUS Message arguments into a hashmap
                let msg_args_hashmap: HashMap<String, Variant<Box<dyn RefArg>>> =
                    iter.read().unwrap();

                // If args have "Metadata" then grab some title, else none
                // this works for like displaying whats playing on youtube
                // for example. Need to look into how much variance there is
                // between all the different messages I want to display.
                if msg_args_hashmap.contains_key("Metadata") {
                    let metadata_variant = msg_args_hashmap.get("Metadata").expect("metadata key");
                    let variant = &metadata_variant.0;
                    let map: &PropMap = arg::cast(variant).unwrap();

                    Some(
                        map.get("xesam:title")
                            .expect("some title")
                            .as_str()
                            .expect("a string")
                            .to_owned(),
                    )
                } else {
                    None
                }
            };
        }

        Notification {
            kind,
            path,
            title,
            summary,
        }
    }
}
