use dbus::{
    arg::{self, PropMap, RefArg, Variant},
    message::MessageType,
    Message,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

/// The Revere Notification type
#[derive(Debug)]
pub struct Notification {
    pub kind: MessageType,
    pub method: String,
    pub sender: String,
    pub destination: String,
    pub serial: Option<u32>,
    pub app_name: String,
    pub id: u64,
    pub icon: Option<String>,
    pub summary: Option<String>,
    pub body: Option<String>,
    pub _actions: Vec<String>,
    pub _hints: HashMap<String, Variant<Box<dyn RefArg>>>,
}
impl From<&Message> for Notification {
    /// Implement DBus Message conversion into a Notification
    fn from(msg: &Message) -> Self {
        let kind = msg.msg_type();
        let method = msg.member().map(|m| m.to_string()).unwrap_or_default();
        let sender = msg.sender().map(|s| s.to_string()).unwrap_or_default();
        let destination = msg.destination().map(|d| d.to_string()).unwrap_or_default();
        let serial = msg.get_serial();

        let mut app_name = String::new();
        let mut id = 0u64;
        let mut icon = None;
        let mut summary = None;
        let mut body = None;
        let mut actions = Vec::new();
        let mut hints = HashMap::new();

        let mut iter = msg.iter_init();

        if let Some(app_name_arg) = iter.get::<String>() {
            app_name = app_name_arg;
        }
        iter.next();

        if let Some(id_arg) = iter.get::<u64>() {
            id = id_arg;
        }
        iter.next();

        if let Some(icon_arg) = iter.get::<String>() {
            icon = Some(icon_arg);
        }
        iter.next();

        if let Some(summary_arg) = iter.get::<String>() {
            summary = Some(summary_arg);
        }
        iter.next();

        if let Some(body_arg) = iter.get::<String>() {
            body = Some(body_arg);
        }
        iter.next();

        if let Some(actions_arg) = iter.get::<Vec<String>>() {
            actions = actions_arg;
        }
        iter.next();

        if let Some(hints_arg) = iter.get::<Variant<Box<dyn RefArg>>>() {
            if let Some(hint_map) = hints_arg.0.as_iter() {
                for entry in hint_map {
                    // Try to downcast each key value pair
                    if let Some((key, value)) = entry.as_iter().and_then(|mut inner| {
                        let key = inner.next()?.as_str()?;
                        let value = inner.next()?.box_clone();
                        Some((key.to_string(), Variant(value)))
                    }) {
                        hints.insert(key, value);
                    }
                }
            }
        }

        Notification {
            kind,
            method,
            sender,
            destination,
            serial,
            app_name,
            id,
            icon,
            summary,
            body,
            _actions: actions,
            _hints: hints,
        }
    }
}
impl Hash for Notification {
    /// Compute a hash for the `Notification`
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.summary.hash(state);
        self.icon.hash(state);
    }
}
