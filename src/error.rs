use std::fmt;

#[derive(Debug)]
pub enum Error {
    FailedToWriteUpdated,
    DisplayFlushError,
    StdIoError(std::io::Error),
    DBusCnxError(dbus::Error),
    DBusMethodError(dbus::MethodErr),
    WaylandCnxError(smithay_client_toolkit::reexports::client::ConnectError),
    WaylandGlobalError(smithay_client_toolkit::reexports::client::GlobalError),
    CairoSurfaceError(cairo::Error),
    CairoBorrowError(cairo::BorrowError),
    CairoIoError(cairo::IoError),
}
impl Error {
    pub fn message(&self) -> String {
        match self {
            Self::FailedToWriteUpdated => String::from("Forgot what this is"),
            Self::DisplayFlushError => String::from("Error: flushing display"),
            Self::StdIoError(_) => String::from("Error: standard output"),
            Self::DBusCnxError(_) => String::from("Error: connecting to D-Bus"),
            Self::DBusMethodError(_) => String::from("Error: issue with D-Bus method"),
            Self::WaylandCnxError(_) => String::from("Error: issue connecting to wayland client"),
            Self::WaylandGlobalError(_) => {
                String::from("Error: issue with a wayland client global binding")
            }
            Self::CairoSurfaceError(e) => format!("Error: issue with cairo surface\n{:?}", e),
            Self::CairoBorrowError(e) => {
                format!("Error: issue with cairo surface data ownership\n{:?}", e)
            }
            Self::CairoIoError(e) => {
                format!("Error: issue with cairo io\n:{e:?}")
            }
        }
    }
}
/// Implement display trait for Error
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
/// Implement error conversion (`std::io::Error` -> `Error`)
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::StdIoError(err)
    }
}
/// Implement error conversion (`dbus::Error` -> `Error`)
impl From<dbus::Error> for Error {
    fn from(err: dbus::Error) -> Error {
        Error::DBusCnxError(err)
    }
}
/// Implement error conversion (`dbus::MethodErr` -> `Error`)
impl From<dbus::MethodErr> for Error {
    fn from(err: dbus::MethodErr) -> Error {
        Error::DBusMethodError(err)
    }
}
/// Implement error conversion (`smithay_client_toolkit::client::ConnectError` -> `Error`)
impl From<smithay_client_toolkit::reexports::client::ConnectError> for Error {
    fn from(err: smithay_client_toolkit::reexports::client::ConnectError) -> Error {
        Error::WaylandCnxError(err)
    }
}
/// Implement error conversion (`smithay_client_toolkit::client::GlobalError` -> `Error`)
impl From<smithay_client_toolkit::reexports::client::GlobalError> for Error {
    fn from(err: smithay_client_toolkit::reexports::client::GlobalError) -> Error {
        Error::WaylandGlobalError(err)
    }
}
/// Implement error conversion (`cairo::Error` -> `Error`)
impl From<cairo::Error> for Error {
    fn from(err: cairo::Error) -> Error {
        Error::CairoSurfaceError(err)
    }
}
/// Implement error conversion (`cairo::BorrowError` -> `Error`)
impl From<cairo::BorrowError> for Error {
    fn from(err: cairo::BorrowError) -> Error {
        Error::CairoBorrowError(err)
    }
}
/// Implement error conversion (`cairo::IoError` -> `Error`)
impl From<cairo::IoError> for Error {
    fn from(err: cairo::IoError) -> Error {
        Error::CairoIoError(err)
    }
}
