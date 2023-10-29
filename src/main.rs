mod error;
mod notification;

use dbus::{blocking::Connection, message::MatchRule};
use error::RevereError;
use notification::Notification;
use std::{result::Result, sync::Arc};

// Hacked up prototype to just set up a connection
// with D-Bus, listen for all messages, and print them.
//
// TODO:
//     * integrate a basic GUI
//     * better DBus Message Parsing
//     * filter D-Bus messages
//     * figure out how end users can config the GUI

pub fn main() -> Result<(), RevereError> {
    // Connect to the DBus session bus
    let bus_cnx = Connection::new_session()?;

    // Wrap the D-Bus connection in Arc for thread safety
    let bus_cnx = Arc::new(bus_cnx);

    // Build a rule listening for all messages on the D-Bus
    // TODO: read more of D-Bus docs for how to actually query this.
    let match_rule = MatchRule::new();
    bus_cnx.add_match(match_rule.clone(), move |_: (), _cnx, msg| {
        println!("Received a message: {:?}", msg);
        let notification = Notification::from(msg);
        println!("{notification:?}");
        true
    })?;

    // Keep it running forever eva
    loop {
        bus_cnx.process(std::time::Duration::from_millis(1000))?;
    }
}
