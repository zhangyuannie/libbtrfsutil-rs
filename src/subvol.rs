use std::{
    ffi::{CString, OsStr},
    num::{NonZeroI64, NonZeroU64},
    os::{raw::c_int, unix::prelude::OsStrExt},
    path::{Path, PathBuf},
    ptr,
    time::{Duration, SystemTime},
};

use uuid::Uuid;

use crate::{Error, FS_TREE_OBJECTID};

/// Information about a Btrfs subvolume.
#[derive(Debug, Clone)]
pub struct SubvolumeInfo(ffi::btrfs_util_subvolume_info);

struct Timespec(ffi::timespec);
impl From<Timespec> for SystemTime {
    fn from(ts: Timespec) -> Self {
        let duration = Duration::new(ts.0.tv_sec as u64, ts.0.tv_nsec as u32);
        SystemTime::UNIX_EPOCH + duration
    }
}

impl SubvolumeInfo {
    pub fn new() -> Self {
        let inner: ffi::btrfs_util_subvolume_info = ffi::btrfs_util_subvolume_info {
            id: 0,
            parent_id: 0,
            dir_id: 0,
            flags: 0,
            uuid: [0u8; 16],
            parent_uuid: [0u8; 16],
            received_uuid: [0u8; 16],
            generation: 0,
            ctransid: 0,
            otransid: 0,
            stransid: 0,
            rtransid: 0,
            ctime: ffi::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            otime: ffi::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            stime: ffi::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            rtime: ffi::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        };
        SubvolumeInfo(inner)
    }

    pub fn as_ptr(&mut self) -> *mut ffi::btrfs_util_subvolume_info {
        &mut self.0
    }

    /// Returns the ID of this subvolume, unique across the filesystem.
    pub fn id(&self) -> u64 {
        self.0.id
    }

    /// Returns the ID of the subvolume which contains this subvolume, or
    /// [`None`] for the root subvolume ([`FS_TREE_OBJECTID`]) or orphaned
    /// subvolumes (i.e., subvolumes which have been deleted but not yet
    /// cleaned up).
    ///
    /// [`FS_TREE_OBJECTID`]: crate::FS_TREE_OBJECTID
    pub fn parent_id(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.parent_id)
    }

    /// Returns the inode number of the directory containing this subvolume in
    /// the parent subvolume, or zero for the root subvolume
    /// ([`FS_TREE_OBJECTID`]) or orphaned subvolumes.
    ///
    /// [`FS_TREE_OBJECTID`]: crate::FS_TREE_OBJECTID
    pub fn dir_id(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.dir_id)
    }

    /// Returns the on-disk root item flags
    pub fn flags(&self) -> u64 {
        self.0.flags
    }

    /// Returns the UUID of this subvolume.
    pub fn uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0.uuid)
    }

    /// Returns the UUID of the subvolume this subvolume is a snapshot of, or
    /// [`None`] if this subvolume is not a snapshot.
    pub fn parent_uuid(&self) -> Option<Uuid> {
        let ret = Uuid::from_bytes(self.0.parent_uuid);
        if ret.is_nil() {
            None
        } else {
            Some(ret)
        }
    }

    /// Returns the UUID of the subvolume this subvolume was received from, or
    /// [`None`] if this subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    pub fn received_uuid(&self) -> Option<Uuid> {
        let ret = Uuid::from_bytes(self.0.received_uuid);
        if ret.is_nil() {
            None
        } else {
            Some(ret)
        }
    }

    /// Returns the transaction ID of the subvolume root.
    pub fn generation(&self) -> u64 {
        self.0.generation
    }

    /// Returns the transaction ID when an inode in this subvolume was last
    /// changed.
    pub fn ctransid(&self) -> u64 {
        self.0.ctransid
    }

    /// Returns the transaction ID when this subvolume was created.
    pub fn otransid(&self) -> u64 {
        self.0.otransid
    }

    /// Returns the transaction ID of the sent subvolume this subvolume was
    /// received from, or [`None`] if this subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    pub fn stransid(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.stransid)
    }

    /// Returns the transaction ID when this subvolume was received, or [`None`]
    /// if this subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    pub fn rtransid(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.rtransid)
    }

    /// Returns the last change time, in seconds since Unix Epoch.
    pub fn ctime(&self) -> i64 {
        self.0.ctime.tv_sec
    }

    /// Returns the last change time, in nanoseconds since [`ctime`].
    ///
    /// [`ctime`]: Self::ctime
    pub fn ctime_nsec(&self) -> i64 {
        self.0.ctime.tv_nsec
    }

    /// Returns the creation time, in seconds since Unix Epoch.
    pub fn otime(&self) -> i64 {
        self.0.otime.tv_sec
    }

    /// Returns the creation time, in nanoseconds since [`otime`].
    ///
    /// [`otime`]: Self::otime
    pub fn otime_nsec(&self) -> i64 {
        self.0.otime.tv_nsec
    }

    /// Returns the send time, in seconds since Unix Epoch.
    ///
    /// Not well-defined, usually [`None`] unless it was set otherwise. This
    /// field is set manually by userspace after a subvolume is received.
    pub fn stime(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.stime.tv_sec)
    }

    /// Returns the send time, in nanoseconds since [`stime`].
    ///
    /// Not well-defined, usually [`None`] unless it was set otherwise. This
    /// field is set manually by userspace after a subvolume is received.
    ///
    /// [`stime`]: Self::stime
    pub fn stime_nsec(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.stime.tv_nsec)
    }

    /// Returns the time when this subvolume was received in seconds since Unix
    /// Epoch, or [`None`] if this subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    pub fn rtime(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.rtime.tv_sec)
    }

    /// Returns the time when this subvolume was received in nanoseconds since
    /// [`rtime`], or [`None`] if this subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    ///
    /// [`rtime`]: Self::rtime
    pub fn rtime_nsec(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.rtime.tv_nsec)
    }

    /// Returns the creation time.
    pub fn created(&self) -> SystemTime {
        Timespec(self.0.otime).into()
    }

    /// Returns the last change time.
    pub fn changed(&self) -> SystemTime {
        Timespec(self.0.ctime).into()
    }

    /// Returns the time when this subvolume was received, or [`None`] if this
    /// subvolume was not received.
    ///
    /// This field is set manually by userspace after a subvolume is received.
    pub fn received(&self) -> Option<SystemTime> {
        if self.0.rtime.tv_sec == 0 && self.0.rtime.tv_nsec == 0 {
            None
        } else {
            Some(Timespec(self.0.ctime).into())
        }
    }
}

impl Default for SubvolumeInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubvolumeIdIterator(*mut ffi::btrfs_util_subvolume_iterator);

/// A builder to create a subvolume iterator
pub struct IterateSubvolume {
    path: CString,
    top: u64,
    post_order: bool,
}

impl IterateSubvolume {
    /// Path in a Btrfs filesystem. This may be any path in the filesystem; it
    /// does not have to refer to a subvolume unless `top` is not provided.
    /// If `top` is not provided, the subvolume ID of `path` is used.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: CString::new(path.as_ref().as_os_str().as_bytes()).unwrap(),
            top: 0,
            post_order: false,
        }
    }

    /// List subvolumes beneath (but not including) the subvolume with this ID.
    /// The returned paths are relative to the subvolume with this ID.
    pub fn top(&mut self, id: u64) -> &mut Self {
        self.top = id;
        self
    }

    // List all subvolumes.
    // This basically sets `top` to `FS_TREE_OBJECTID`.
    pub fn all(&mut self) -> &mut Self {
        self.top = FS_TREE_OBJECTID;
        self
    }

    /// Use post order traversal
    pub fn post_order(&mut self) -> &mut Self {
        self.post_order = true;
        self
    }

    /// Use pre order traversal (default)
    pub fn pre_order(&mut self) -> &mut Self {
        self.post_order = false;
        self
    }

    /// Returns an iterator to iterate over subvolume IDs
    pub fn iter_with_id(&self) -> Result<SubvolumeIdIterator, Error> {
        let mut flags: c_int = 0;
        if self.post_order {
            flags |= ffi::BTRFS_UTIL_SUBVOLUME_ITERATOR_POST_ORDER as c_int;
        }

        let mut iter: *mut ffi::btrfs_util_subvolume_iterator = ptr::null_mut();
        unsafe {
            let errcode = ffi::btrfs_util_create_subvolume_iterator(
                self.path.as_ptr(),
                self.top,
                flags,
                &mut iter,
            );
            if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
                return Err(Error::new(errcode));
            }
        }
        Ok(SubvolumeIdIterator(iter))
    }

    /// Returns an iterator to iterate over subvolume info
    pub fn iter_with_info(&self) -> Result<SubvolumeInfoIterator, Error> {
        Ok(self.iter_with_id()?.into())
    }
}

/// The given pointer will be freed
unsafe fn c_char_ptr_to_path(ptr: *mut std::os::raw::c_char) -> PathBuf {
    let c_str = std::ffi::CStr::from_ptr(ptr);
    let os_str = OsStr::from_bytes(c_str.to_bytes());
    let ret = PathBuf::from(os_str);
    libc::free(ptr as *mut libc::c_void);

    ret
}

impl Iterator for SubvolumeIdIterator {
    type Item = Result<(PathBuf, NonZeroU64), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut path_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut id: u64 = 0;
        let errcode =
            unsafe { ffi::btrfs_util_subvolume_iterator_next(self.0, &mut path_ptr, &mut id) };
        match errcode {
            ffi::btrfs_util_error::BTRFS_UTIL_OK => {
                let path = unsafe { c_char_ptr_to_path(path_ptr) };
                Some(Ok((path, NonZeroU64::new(id).unwrap())))
            }
            ffi::btrfs_util_error::BTRFS_UTIL_ERROR_STOP_ITERATION => None,
            _ => Some(Err(Error::new(errcode))),
        }
    }
}

impl Drop for SubvolumeIdIterator {
    fn drop(&mut self) {
        unsafe {
            ffi::btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}

impl From<SubvolumeInfoIterator> for SubvolumeIdIterator {
    fn from(iter: SubvolumeInfoIterator) -> Self {
        iter.0
    }
}

impl From<SubvolumeIdIterator> for SubvolumeInfoIterator {
    fn from(iter: SubvolumeIdIterator) -> Self {
        Self(iter)
    }
}

pub struct SubvolumeInfoIterator(SubvolumeIdIterator);

impl Iterator for SubvolumeInfoIterator {
    type Item = Result<(PathBuf, SubvolumeInfo), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut path_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut info = SubvolumeInfo::new();
        let errcode = unsafe {
            ffi::btrfs_util_subvolume_iterator_next_info(self.0 .0, &mut path_ptr, &mut info.0)
        };
        match errcode {
            ffi::btrfs_util_error::BTRFS_UTIL_OK => {
                let path = unsafe { c_char_ptr_to_path(path_ptr) };
                Some(Ok((path, info)))
            }
            ffi::btrfs_util_error::BTRFS_UTIL_ERROR_STOP_ITERATION => None,
            _ => Some(Err(Error::new(errcode))),
        }
    }
}

/// Gets the path of the subvolume relative to the filesystem root.
///
/// This requires appropriate privilege (`CAP_SYS_ADMIN`).
#[inline]
pub fn subvolume_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
    subvolume_path_with_id(path, 0)
}

/// Gets the path of the subvolume with a given ID relative to the filesystem root.
///
/// This requires appropriate privilege (`CAP_SYS_ADMIN`).
pub fn subvolume_path_with_id<P: AsRef<Path>>(path: P, id: u64) -> Result<PathBuf, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut ret_path_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
    unsafe {
        let errcode = ffi::btrfs_util_subvolume_path(cpath.as_ptr(), id, &mut ret_path_ptr);
        if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
            return Err(Error::new(errcode));
        }
        let path = c_char_ptr_to_path(ret_path_ptr);

        Ok(path)
    }
}
