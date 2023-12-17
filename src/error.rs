use std::fmt;

#[derive(Debug)]
pub enum RevereError {
    FailedToWriteUpdated,
    DisplayFlushError,
    StdIoError(std::io::Error),
    DBusCnxError(dbus::Error),
    DBusMethodError(dbus::MethodErr),
}
// Implement display trait for RevereError
impl fmt::Display for RevereError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Think of more meaningful display messages
        write!(f, "error")
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
