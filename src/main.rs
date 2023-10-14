mod error;
use error::RevereError;
use std::result::Result;
use dbus::{channel::MatchingReceiver, message::MatchRule};
use dbus_tokio::connection;
use tokio::task;
use std::sync::Arc;

// Hacked up prototype to just set up a connection
// with D-Bus, listen for all messages, and print them.
//
// TODO: 
//     * integrate a basic GUI
//     * filter D-Bus messages
//     * figure out how end users can config the GUI

#[tokio::main]
pub async fn main() -> Result<(), RevereError> {
    // Connect to the DBus session bus
    let (resrc, bus_cnx) = connection::new_session_sync()?;
    task::spawn(async {
        let err = resrc.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    // Wrap the D-Bus connection in Arc for thread safety
    let bus_cnx = Arc::new(bus_cnx);

    // Build a rule listening for all messages on the D-Bus
    // TODO: read more of D-Bus docs for how to actually query this.
    let match_rule = MatchRule::new();
    bus_cnx.add_match(match_rule.clone()).await?;

    // Set a callback function to call when D-Bus recieves
    // a message matching our rule above (`match_rule`)
    bus_cnx.start_receive(match_rule, Box::new(|msg, _| {
        // Just print out the message for now
        println!("Received a message: {msg:?}");
        true
    }));

    // Keep it running foreva eva
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
