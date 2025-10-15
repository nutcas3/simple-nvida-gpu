pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NoMem,
    Invalid,
    NoDevice,
    Timeout,
    Io,
    Again,
    Busy,
    NotSupported,
    PermissionDenied,
}

impl Error {
    pub fn to_errno(self) -> i32 {
        match self {
            Error::NoMem => -12,
            Error::Invalid => -22,
            Error::NoDevice => -19,
            Error::Timeout => -110,
            Error::Io => -5,
            Error::Again => -11,
            Error::Busy => -16,
            Error::NotSupported => -95,
            Error::PermissionDenied => -13,
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::NoMem => write!(f, "Out of memory"),
            Error::Invalid => write!(f, "Invalid argument"),
            Error::NoDevice => write!(f, "No such device"),
            Error::Timeout => write!(f, "Timeout"),
            Error::Io => write!(f, "I/O error"),
            Error::Again => write!(f, "Try again"),
            Error::Busy => write!(f, "Device busy"),
            Error::NotSupported => write!(f, "Not supported"),
            Error::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}
