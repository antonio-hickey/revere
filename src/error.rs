use std::fmt;

#[derive(Debug)]
pub enum RevereError {
    FailedToWriteUpdated,
    DisplayFlushError,
    StdIoError(std::io::Error),
    DBusCnxError(dbus::Error),
    DBusMethodError(dbus::MethodErr),
    WaylandCnxError(smithay_client_toolkit::reexports::client::ConnectError),
    WaylandGlobalError(smithay_client_toolkit::reexports::client::GlobalError),
}
impl RevereError {
    pub fn message(&self) -> String {
        match self {
            Self::FailedToWriteUpdated => String::from("Forgot what this is"),
            Self::DisplayFlushError => String::from("Error: flushing display"),
            Self::StdIoError(_) => String::from("Error: standard output"),
            Self::DBusCnxError(_) => String::from("Error: connecting to D-Bus"),
            Self::DBusMethodError(_) => String::from("Error: issue with D-Bus method"),
            Self::WaylandCnxError(_) => String::from("Error: issue connecting to wayland client"),
            Self::WaylandGlobalError(_) => String::from("Error: issue with a wayland client global binding"),
        }
    }
}
// Implement display trait for RevereError
impl fmt::Display for RevereError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
/// Implement error conversion (`std::io::Error` -> `RevereError`)
impl From<std::io::Error> for RevereError {
    fn from(err: std::io::Error) -> RevereError {
        RevereError::StdIoError(err)
    }
}
/// Implement error conversion (`dbus::Error` -> `RevereError`)
impl From<dbus::Error> for RevereError {
    fn from(err: dbus::Error) -> RevereError {
        RevereError::DBusCnxError(err)
    }
}
/// Implement error conversion (`dbus::MethodErr` -> `RevereError`)
impl From<dbus::MethodErr> for RevereError {
    fn from(err: dbus::MethodErr) -> RevereError {
        RevereError::DBusMethodError(err)
    }
}
/// Implement error conversion (`smithay_client_toolkit::client::ConnectError` -> `RevereError`)
impl From<smithay_client_toolkit::reexports::client::ConnectError> for RevereError {
    fn from(err: smithay_client_toolkit::reexports::client::ConnectError) -> RevereError {
        RevereError::WaylandCnxError(err)
    }
}
/// Implement error conversion (`smithay_client_toolkit::client::GlobalError` -> `RevereError`)
impl From<smithay_client_toolkit::reexports::client::GlobalError> for RevereError {
    fn from(err: smithay_client_toolkit::reexports::client::GlobalError) -> RevereError {
        RevereError::WaylandGlobalError(err)
    }
}
