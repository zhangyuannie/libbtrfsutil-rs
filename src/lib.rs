mod error;
mod qgroup;
mod subvol;

use std::{
    ffi::CString,
    os::{raw::c_int, unix::prelude::OsStrExt},
    path::Path,
};

pub use error::{Error, ErrorKind};
pub use qgroup::QgroupInherit;
pub use subvol::*;
pub const FS_TREE_OBJECTID: u64 = 5;

/// Forces a sync on a Btrfs filesystem containing the `path`.
pub fn sync<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let errcode = unsafe { ffi::btrfs_util_sync(cpath.as_ptr()) };
    if errcode == ffi::btrfs_util_error::BTRFS_UTIL_OK {
        Ok(())
    } else {
        Err(Error::new(errcode))
    }
}

/// Returns whether the given `path` is a Btrfs subvolume.
pub fn is_subvolume<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let errcode = unsafe { ffi::btrfs_util_is_subvolume(cpath.as_ptr()) };
    match errcode {
        ffi::btrfs_util_error::BTRFS_UTIL_OK => Ok(true),
        ffi::btrfs_util_error::BTRFS_UTIL_ERROR_NOT_SUBVOLUME
        | ffi::btrfs_util_error::BTRFS_UTIL_ERROR_NOT_BTRFS => Ok(false),
        _ => Err(Error::new(errcode)),
    }
}

/// Gets the ID of the subvolume containing the `path`.
pub fn subvolume_id<P: AsRef<Path>>(path: P) -> Result<u64, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut ret: u64 = 0;
    let errcode = unsafe { ffi::btrfs_util_subvolume_id(cpath.as_ptr(), &mut ret) };
    if errcode == ffi::btrfs_util_error::BTRFS_UTIL_OK {
        Ok(ret)
    } else {
        Err(Error::new(errcode))
    }
}

/// Gets information about the subvolume with the given `id` on the filesystem containing the `path`.
///
/// This requires appropriate privilege (`CAP_SYS_ADMIN`).
pub fn subvolume_info_with_id<P: AsRef<Path>>(path: P, id: u64) -> Result<SubvolumeInfo, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut out = SubvolumeInfo::new();
    unsafe {
        let errcode = ffi::btrfs_util_subvolume_info(cpath.as_ptr(), id, out.as_ptr());
        if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
            return Err(Error::new(errcode));
        }
    }
    Ok(out)
}

/// Gets information about the subvolume at the given `path`.
///
/// This requires appropriate privilege (`CAP_SYS_ADMIN`) unless the kernel supports
/// `BTRFS_IOC_GET_SUBVOL_INFO` (kernel >= 4.18).
pub fn subvolume_info<P: AsRef<Path>>(path: P) -> Result<SubvolumeInfo, Error> {
    subvolume_info_with_id(path, 0)
}

/// Returns whether a subvolume is read-only.
pub fn subvolume_read_only<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut ret: bool = false;

    let errcode = unsafe { ffi::btrfs_util_get_subvolume_read_only(cpath.as_ptr(), &mut ret) };
    if errcode == ffi::btrfs_util_error::BTRFS_UTIL_OK {
        Ok(ret)
    } else {
        Err(Error::new(errcode))
    }
}

/// Set whether a subvolume is read-only.
///
/// This requires appropriate privilege (CAP_SYS_ADMIN).
pub fn set_subvolume_read_only<P: AsRef<Path>>(path: P, read_only: bool) -> Result<(), Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let errcode = unsafe { ffi::btrfs_util_set_subvolume_read_only(cpath.as_ptr(), read_only) };
    if errcode == ffi::btrfs_util_error::BTRFS_UTIL_OK {
        Ok(())
    } else {
        Err(Error::new(errcode))
    }
}

/// Options to delete subvolumes
pub struct DeleteSubvolumeOptions {
    recursive: bool,
}

impl DeleteSubvolumeOptions {
    pub fn new() -> Self {
        Self { recursive: false }
    }
    /// When true, delete subvolumes beneath the given subvolume before
    /// attempting to delete the given subvolume.
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Deletes a subvolume or snapshot.
    pub fn delete<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut flags: c_int = 0;
        if self.recursive {
            flags |= ffi::BTRFS_UTIL_DELETE_SUBVOLUME_RECURSIVE as c_int;
        }
        let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
        unsafe {
            let errcode = ffi::btrfs_util_delete_subvolume(cpath.as_ptr(), flags);
            if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
                return Err(Error::new(errcode));
            }
        }
        Ok(())
    }
}

/// Delete a subvolume. See [`DeleteSubvolumeOptions`] for more options.
pub fn delete_subvolume<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    DeleteSubvolumeOptions::new().delete(path)
}

/// Options to create subvolumes
pub struct CreateSubvolumeOptions {
    qgroup: Option<QgroupInherit>,
}

impl CreateSubvolumeOptions {
    pub fn new() -> Self {
        Self { qgroup: None }
    }

    pub fn qgroup(&mut self, qgroup: Option<QgroupInherit>) -> &mut Self {
        self.qgroup = qgroup;
        self
    }

    /// Creates a new subvolume.
    pub fn create<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
        let flags: c_int = 0;

        let cqgroup: *mut ffi::btrfs_util_qgroup_inherit = if let Some(qg) = &self.qgroup {
            qg.as_ptr()
        } else {
            std::ptr::null_mut()
        };

        let errcode = unsafe {
            ffi::btrfs_util_create_subvolume(cpath.as_ptr(), flags, std::ptr::null_mut(), cqgroup)
        };
        if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
            Err(Error::new(errcode))
        } else {
            Ok(())
        }
    }
}

/// Creates a new subvolume. See [`CreateSubvolumeOptions`] for more options.
pub fn create_subvolume<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    CreateSubvolumeOptions::new().create(path)
}

/// Options to create snapshots
pub struct CreateSnapshotOptions {
    qgroup: Option<QgroupInherit>,
    readonly: bool,
    recursive: bool,
}

impl CreateSnapshotOptions {
    pub fn new() -> Self {
        Self {
            qgroup: None,
            readonly: false,
            recursive: false,
        }
    }

    pub fn qgroup(&mut self, qgroup: Option<QgroupInherit>) -> &mut Self {
        self.qgroup = qgroup;
        self
    }

    pub fn readonly(&mut self, readonly: bool) -> &mut Self {
        self.readonly = readonly;
        self
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Creates a new snapshot from a source subvolume.
    pub fn create<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        source: P,
        path: Q,
    ) -> Result<(), Error> {
        let csource = CString::new(source.as_ref().as_os_str().as_bytes()).unwrap();
        let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();

        let mut flags: c_int = 0;
        if self.readonly {
            flags |= ffi::BTRFS_UTIL_CREATE_SNAPSHOT_READ_ONLY as c_int;
        }
        if self.recursive {
            flags |= ffi::BTRFS_UTIL_CREATE_SNAPSHOT_RECURSIVE as c_int;
        }

        let cqgroup: *mut ffi::btrfs_util_qgroup_inherit = if let Some(qg) = &self.qgroup {
            qg.as_ptr()
        } else {
            std::ptr::null_mut()
        };
        unsafe {
            let errcode = ffi::btrfs_util_create_snapshot(
                csource.as_ptr(),
                cpath.as_ptr(),
                flags,
                std::ptr::null_mut(),
                cqgroup,
            );
            if errcode != ffi::btrfs_util_error::BTRFS_UTIL_OK {
                return Err(Error::new(errcode));
            }
        }
        Ok(())
    }
}
