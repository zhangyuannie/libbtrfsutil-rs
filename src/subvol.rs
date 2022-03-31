use std::{
    ffi::{CString, OsStr},
    num::{NonZeroI64, NonZeroU64, TryFromIntError},
    os::{raw::c_int, unix::prelude::OsStrExt},
    path::{Path, PathBuf},
    ptr,
    time::{Duration, SystemTime},
};

use bitflags::bitflags;
use uuid::Uuid;

use crate::Error;

#[derive(Debug, Clone)]
pub struct SubvolumeInfo(ffi::btrfs_util_subvolume_info);

struct Timespec(ffi::timespec);
impl TryInto<SystemTime> for Timespec {
    type Error = TryFromIntError;

    fn try_into(self) -> Result<SystemTime, Self::Error> {
        let duration = Duration::new(self.0.tv_sec.try_into()?, self.0.tv_nsec.try_into()?);
        Ok(SystemTime::UNIX_EPOCH + duration)
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

    pub fn id(&self) -> u64 {
        self.0.id
    }

    pub fn parent_id(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.parent_id)
    }

    pub fn dir_id(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.dir_id)
    }

    pub fn flags(&self) -> u64 {
        self.0.flags
    }

    pub fn uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0.uuid)
    }

    pub fn parent_uuid(&self) -> Option<Uuid> {
        let ret = Uuid::from_bytes(self.0.parent_uuid);
        if ret.is_nil() {
            None
        } else {
            Some(ret)
        }
    }

    pub fn received_uuid(&self) -> Option<Uuid> {
        let ret = Uuid::from_bytes(self.0.received_uuid);
        if ret.is_nil() {
            None
        } else {
            Some(ret)
        }
    }

    pub fn generation(&self) -> u64 {
        self.0.generation
    }

    pub fn ctransid(&self) -> u64 {
        self.0.ctransid
    }

    pub fn otransid(&self) -> u64 {
        self.0.otransid
    }

    pub fn stransid(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.stransid)
    }

    pub fn rtransid(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.rtransid)
    }

    /// Returns the last change time, in seconds since Unix Epoch.
    pub fn ctime(&self) -> i64 {
        self.0.ctime.tv_sec
    }

    /// Returns the last change time, in nanoseconds since `ctime`.
    pub fn ctime_nsec(&self) -> i64 {
        self.0.ctime.tv_nsec
    }

    /// Returns the creation time, in seconds since Unix Epoch.
    pub fn otime(&self) -> i64 {
        self.0.otime.tv_sec
    }

    /// Returns the creation time, in nanoseconds since `otime`.
    pub fn otime_nsec(&self) -> i64 {
        self.0.otime.tv_nsec
    }

    pub fn stime(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.stime.tv_sec)
    }

    pub fn stime_nsec(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.stime.tv_nsec)
    }

    /// Returns the receipt time, in seconds since Unix Epoch.
    pub fn rtime(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.rtime.tv_sec)
    }

    /// Returns the receipt time, in nanoseconds since `rtime`.
    pub fn rtime_nsec(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(self.0.rtime.tv_nsec)
    }

    /// Returns the creation time
    pub fn created(&self) -> SystemTime {
        Timespec(self.0.otime).try_into().unwrap()
    }

    /// Returns the last change time
    pub fn changed(&self) -> SystemTime {
        Timespec(self.0.ctime).try_into().unwrap()
    }

    pub fn received(&self) -> Option<SystemTime> {
        if self.0.rtime.tv_sec == 0 && self.0.rtime.tv_nsec == 0 {
            None
        } else {
            Some(Timespec(self.0.ctime).try_into().unwrap())
        }
    }
}

impl Default for SubvolumeInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubvolumeIterator(*mut ffi::btrfs_util_subvolume_iterator);

bitflags! {
    #[derive(Default)]
    pub struct SubvolumeIteratorFlags: c_int {
        const POST_ORDER = ffi::BTRFS_UTIL_SUBVOLUME_ITERATOR_POST_ORDER as c_int;
    }
}

impl SubvolumeIterator {
    pub fn new<P: AsRef<Path>>(
        path: P,
        top: Option<NonZeroU64>,
        flags: SubvolumeIteratorFlags,
    ) -> Result<Self, Error> {
        let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
        let ctop = top.map_or(0, |i| i.get());
        let cflags = flags.bits();
        let mut iter: *mut ffi::btrfs_util_subvolume_iterator = ptr::null_mut();
        unsafe {
            let errcode =
                ffi::btrfs_util_create_subvolume_iterator(cpath.as_ptr(), ctop, cflags, &mut iter);
            if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
                return Err(errcode.into());
            }
        }
        Ok(SubvolumeIterator(iter))
    }
}

fn c_char_ptr_to_path(ptr: *mut std::os::raw::c_char) -> PathBuf {
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    let os_str = OsStr::from_bytes(c_str.to_bytes());
    let ret = PathBuf::from(os_str);
    unsafe {
        libc::free(ptr as *mut libc::c_void);
    }
    ret
}

impl Iterator for SubvolumeIterator {
    type Item = Result<(PathBuf, NonZeroU64), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut path_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut id: u64 = 0;
        let errcode =
            unsafe { ffi::btrfs_util_subvolume_iterator_next(self.0, &mut path_ptr, &mut id) };
        match errcode {
            ffi::btrfs_util_error_BTRFS_UTIL_OK => {
                let path = c_char_ptr_to_path(path_ptr);
                Some(Ok((path, NonZeroU64::new(id).unwrap())))
            }
            ffi::btrfs_util_error_BTRFS_UTIL_ERROR_STOP_ITERATION => None,
            _ => Some(Err(errcode.into())),
        }
    }
}

impl Drop for SubvolumeIterator {
    fn drop(&mut self) {
        unsafe {
            ffi::btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}

impl From<SubvolumeInfoIterator> for SubvolumeIterator {
    fn from(iter: SubvolumeInfoIterator) -> Self {
        iter.0
    }
}

impl From<SubvolumeIterator> for SubvolumeInfoIterator {
    fn from(iter: SubvolumeIterator) -> Self {
        Self(iter)
    }
}

pub struct SubvolumeInfoIterator(SubvolumeIterator);

impl SubvolumeInfoIterator {
    pub fn new<P: AsRef<Path>>(
        path: P,
        top: Option<NonZeroU64>,
        flags: SubvolumeIteratorFlags,
    ) -> Result<Self, Error> {
        SubvolumeIterator::new(path, top, flags).map(|iter| Self(iter))
    }
}

impl Iterator for SubvolumeInfoIterator {
    type Item = Result<(PathBuf, SubvolumeInfo), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut path_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut info = SubvolumeInfo::new();
        let errcode = unsafe {
            ffi::btrfs_util_subvolume_iterator_next_info(self.0 .0, &mut path_ptr, &mut info.0)
        };
        match errcode {
            ffi::btrfs_util_error_BTRFS_UTIL_OK => {
                let path = c_char_ptr_to_path(path_ptr);
                Some(Ok((path, info)))
            }
            ffi::btrfs_util_error_BTRFS_UTIL_ERROR_STOP_ITERATION => None,
            _ => Some(Err(errcode.into())),
        }
    }
}
