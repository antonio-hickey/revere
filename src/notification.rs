use dbus::arg::{self, PropMap, RefArg, Variant};
use dbus::message::MessageType;
use dbus::Message;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// The Revere Notification type
#[derive(Debug)]
pub struct Notification {
    pub kind: MessageType,
    pub path: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub image: Option<String>,
}
impl From<&Message> for Notification {
    /// Implement DBus Message conversion into a Notification
    fn from(msg: &Message) -> Self {
        let kind = msg.msg_type();
        let path = msg.path().unwrap().to_string();
        let (mut title, summary, mut image) = (None, None, None);

        // Parse out a title/action from a PropertiesChanged member message
        if msg.member().expect("some interface member").to_string() == *"PropertiesChanged" {
            // Iterate over the message arguments
            let mut iter = msg.iter_init();
            // Skip the first iteration
            iter.next();

            // Parse the D-BUS Message arguments into a hashmap
            let msg_args_hashmap: HashMap<String, Variant<Box<dyn RefArg>>> = iter.read().unwrap();

            // If args have "Metadata" then grab some data, else none
            // this works for like displaying whats playing on youtube
            // for example. Need to look into how much variance there is
            // between all the different messages I want to display.
            if msg_args_hashmap.contains_key("Metadata") {
                let metadata_variant = msg_args_hashmap.get("Metadata").expect("metadata key");
                let variant = &metadata_variant.0;
                let map: &PropMap = arg::cast(variant).unwrap();

                title = Some(
                    map.get("xesam:title")
                        .expect("some title")
                        .as_str()
                        .expect("a string")
                        .to_owned(),
                );

                image = map
                    .get("mpris:artUrl")
                    .map(|value| value.as_str())
                    .and_then(|value| value.map(|s| s.to_owned().replace("file://", "")));
            }
        }

        Notification {
            kind,
            path,
            title,
            summary,
            image,
        }
    }
}
impl Hash for Notification {
    /// Compute a hash for the `Notification`
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.image.hash(state);
    }
}
