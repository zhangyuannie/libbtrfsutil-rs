use std::{
    ffi::CStr,
    fmt::{self, Display},
    io, str,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ErrorKind(ffi::btrfs_util_error::Type);

impl ErrorKind {
    pub const OK: ErrorKind = ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_OK);
    pub const STOP_ITERATION: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_STOP_ITERATION);
    pub const NO_MEMORY: ErrorKind = ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_NO_MEMORY);
    pub const INVALID_ARGUMENT: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_INVALID_ARGUMENT);
    pub const NOT_BTRFS: ErrorKind = ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_NOT_BTRFS);
    pub const NOT_SUBVOLUME: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_NOT_SUBVOLUME);
    pub const SUBVOLUME_NOT_FOUND: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SUBVOLUME_NOT_FOUND);
    pub const OPEN_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_OPEN_FAILED);
    pub const RMDIR_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_RMDIR_FAILED);
    pub const UNLINK_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_UNLINK_FAILED);
    pub const STAT_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_STAT_FAILED);
    pub const STATFS_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_STATFS_FAILED);
    pub const SEARCH_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SEARCH_FAILED);
    pub const INO_LOOKUP_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_INO_LOOKUP_FAILED);
    pub const SUBVOL_GETFLAGS_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SUBVOL_GETFLAGS_FAILED);
    pub const SUBVOL_SETFLAGS_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SUBVOL_SETFLAGS_FAILED);
    pub const SUBVOL_CREATE_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SUBVOL_CREATE_FAILED);
    pub const SNAP_CREATE_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SNAP_CREATE_FAILED);
    pub const SNAP_DESTROY_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SNAP_DESTROY_FAILED);
    pub const DEFAULT_SUBVOL_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_DEFAULT_SUBVOL_FAILED);
    pub const SYNC_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_SYNC_FAILED);
    pub const START_SYNC_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_START_SYNC_FAILED);
    pub const WAIT_SYNC_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_WAIT_SYNC_FAILED);
    pub const GET_SUBVOL_INFO_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_GET_SUBVOL_INFO_FAILED);
    pub const GET_SUBVOL_ROOTREF_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_GET_SUBVOL_ROOTREF_FAILED);
    pub const INO_LOOKUP_USER_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_INO_LOOKUP_USER_FAILED);
    pub const FS_INFO_FAILED: ErrorKind =
        ErrorKind(ffi::btrfs_util_error::BTRFS_UTIL_ERROR_FS_INFO_FAILED);
}

impl From<ErrorKind> for u32 {
    fn from(kind: ErrorKind) -> Self {
        kind.0 as u32
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
    pub(crate) fn new(kind: ffi::btrfs_util_error::Type) -> Self {
        let errno = io::Error::last_os_error().raw_os_error().unwrap();
        Error {
            kind: ErrorKind(kind),
            errno: Errno(errno),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn errno(&self) -> i32 {
        self.errno.0
    }

    pub fn os_error(&self) -> io::Error {
        io::Error::from_raw_os_error(self.errno.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_ptr = unsafe { ffi::btrfs_util_strerror(self.kind.0) };
        if str_ptr.is_null() {
            write!(f, "unknown libbtrfsutil error {}", self.kind.0)
        } else {
            let slice = unsafe { CStr::from_ptr(str_ptr).to_bytes() };
            let slice = str::from_utf8(slice).unwrap();
            let first_char = slice.chars().next().unwrap().to_ascii_lowercase();
            write!(f, "{}{}", first_char, &slice[1..])
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&self.errno)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use crate::Error;

    #[test]
    fn test_display() {
        unsafe {
            let err = Error::new(4);
            let received = err.to_string();
            let expected_ptr = ffi::btrfs_util_strerror(4);
            let expected = CStr::from_ptr(expected_ptr).to_str().unwrap().to_owned();
            let expected_lower = expected.to_ascii_lowercase();
            assert_eq!(received[..1], expected_lower[..1]);
            assert_eq!(received[1..], expected[1..]);
        }
    }

    #[test]
    fn test_display_unknown() {
        let err = Error::new(99);
        let received = err.to_string();
        assert_eq!(received, "unknown libbtrfsutil error 99");
    }
}
