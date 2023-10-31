mod error;
mod notification;

use dbus::{blocking::Connection, message::MatchRule};
use error::RevereError;
use notification::Notification;
use std::{
    ffi::CString,
    ptr,
    result::Result,
    sync::Arc,
    time::{Duration, Instant},
};
use x11::xlib;

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

        unsafe {
            // Open a connection to the X server
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!("Could not open display.");
            }

            // Define some variables for the screen environment
            let screen = xlib::XDefaultScreen(display);
            let root = xlib::XRootWindow(display, screen);
            let screen_width = xlib::XDisplayWidth(display, screen);
            let _screen_height = xlib::XDisplayHeight(display, screen);

            // Calculate the top-right corner coordinates
            let x = screen_width - 1650;
            let y = 575;

            // Create a basic window
            let window = xlib::XCreateSimpleWindow(
                display,
                root,
                x,   // x-axis screen postion
                y,   // y-axis screen position
                200, // Width
                50,  // Height
                1,
                xlib::XBlackPixel(display, screen), // Border
                xlib::XWhitePixel(display, screen), // Background
            );

            // Fetch atoms for _NET_WM_WINDOW_TYPE and _NET_WM_WINDOW_TYPE_NOTIFICATION
            let wm_window_type =
                xlib::XInternAtom(display, "_NET_WM_WINDOW_TYPE\0".as_ptr() as *const i8, 0);
            let wm_window_type_notification = xlib::XInternAtom(
                display,
                "_NET_WM_WINDOW_TYPE_NOTIFICATION\0".as_ptr() as *const i8,
                0,
            );

            // Set the _NET_WM_WINDOW_TYPE property to _NET_WM_WINDOW_TYPE_NOTIFICATION
            xlib::XChangeProperty(
                display,
                window,
                wm_window_type,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                &wm_window_type_notification as *const u64 as *const u8,
                1,
            );
            xlib::XSelectInput(display, window, xlib::ExposureMask);
            xlib::XMapWindow(display, window);
            xlib::XFlush(display);

            // Define a graphics context for our window
            let gc = xlib::XCreateGC(display, window, 0, ptr::null_mut());

            // Build the window for 3 seconds
            let start_time = Instant::now();
            while start_time.elapsed() < Duration::from_secs(3) {
                if xlib::XPending(display) > 0 {
                    // X11 Event
                    let mut ev = xlib::XEvent { pad: [0; 24] };
                    xlib::XNextEvent(display, &mut ev);

                    // Match the X11 event to some functionality
                    match ev.get_type() {
                        xlib::Expose => {
                            let string = notification.title.clone();
                            let c_string =
                                CString::new(string.as_str()).expect("Failed to create CString");
                            xlib::XDrawString(
                                display,
                                window,
                                gc,                  // graphic context
                                10,                  // x coordinate
                                20,                  // y coordinate
                                c_string.as_ptr(),   // raw pointer to our notification string data
                                string.len() as i32, // chars in our notification string data
                            );
                            xlib::XFlush(display);
                        }
                        _ => {}
                    }
                }
            }
            // Destroy & close the window
            xlib::XDestroyWindow(display, window);
            xlib::XCloseDisplay(display);
        }
        true
    })?;

    // Keep it running forever eva
    loop {
        bus_cnx.process(std::time::Duration::from_millis(1000))?;
    }
}
