use crate::{config::Config, window::NotificationWindow};
use dbus::{
    arg::{RefArg, Variant},
    message::MessageType,
    Message,
};
use std::{
    collections::HashMap,
    fs::File,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

/// The default notification icon for Revere
static DEFAULT_ICON_PNG: &[u8] = include_bytes!("../assets/notification-icon.png");

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
impl Notification {
    /// Display the [`Notification`] in a new [`NotificationWindow`].
    pub fn display_in_new_window(&self, config: &Config) {
        // Extract the icon image from the notification
        let icon = self
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
        let mut notification_window = NotificationWindow::try_new(&config.window)
            .expect("Failed to crate notification window");

        // Render the notification window for some time duration (default: 3 seconds)
        let start_time = Instant::now();
        let duration_time = Duration::from_secs(config.window.duration as u64);
        while start_time.elapsed() < duration_time {
            let elapsed = start_time.elapsed().as_secs_f64();
            let duration = duration_time.as_secs_f64();
            let completion_percent = elapsed / duration * 100.00;

            // Try to handle the dispatching of events on the notification window
            if let Err(e) = notification_window
                .event_queue
                .dispatch(&mut (), |_, _, _| {})
            {
                eprintln!("Failed to dispatch event: {e:?}");
            }

            // Try to render the notification window
            if let Err(e) = notification_window.draw(
                &self.summary.clone().unwrap_or_default(),
                &self.body.clone().unwrap_or_default(),
                &image_surface,
                &config.window,
                completion_percent,
            ) {
                eprintln!("Failed to draw notification window: {e:?}");
            }
        }

        // Flush the display of the notification window
        notification_window.flush_display().ok();
    }
}
