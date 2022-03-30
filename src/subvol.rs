use std::{
    num::{NonZeroI64, NonZeroU64, TryFromIntError},
    time::{Duration, SystemTime},
};

use uuid::Uuid;

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

impl Iterator for SubvolumeIterator {
    type Item = SubvolumeInfo;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Drop for SubvolumeIterator {
    fn drop(&mut self) {
        unsafe {
            ffi::btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}
