use std::{
    ffi::CStr,
    fmt::{self, Display},
    io,
};

pub use ffi::BtrfsUtilError as ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Errno(i32);

impl Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        io::Error::from_raw_os_error(self.0).fmt(f)
    }
}
impl std::error::Error for Errno {}

impl From<Errno> for io::Error {
    fn from(e: Errno) -> Self {
        io::Error::from_raw_os_error(e.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    kind: ErrorKind,
    errno: Errno,
}

impl Error {
    /// This Error should be created immediately after a function call from libbtrfsutil to capture errno.
    #[inline]
    pub(crate) fn new(kind: ErrorKind) -> Self {
        let errno = io::Error::last_os_error().raw_os_error().unwrap();
        Error {
            kind,
            errno: Errno(errno),
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the underlying errno.
    pub fn errno(&self) -> i32 {
        self.errno.0
    }

    /// Returns the corresponding [`io::Error`] for the underlying errno.
    pub fn os_error(&self) -> io::Error {
        io::Error::from_raw_os_error(self.errno.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_ptr = unsafe { ffi::btrfs_util_strerror(self.kind) };
        if str_ptr.is_null() {
            write!(f, "Unknown libbtrfsutil error {}", self.kind.0)
        } else {
            let message = unsafe { CStr::from_ptr(str_ptr) }.to_string_lossy();
            f.write_str(&message)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.errno)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use crate::{Error, ErrorKind};

    #[test]
    fn test_display() {
        unsafe {
            let kind = ErrorKind::NOT_BTRFS;
            let err = Error::new(kind);
            let received = err.to_string();
            let expected_ptr = ffi::btrfs_util_strerror(kind);
            let expected = CStr::from_ptr(expected_ptr).to_str().unwrap().to_owned();
            assert_eq!(received, expected);
        }
    }

    #[test]
    fn test_display_unknown() {
        let err = Error::new(ErrorKind(99));
        let received = err.to_string();
        assert_eq!(received, "Unknown libbtrfsutil error 99");
    }
}
