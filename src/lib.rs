mod error;
mod qgroup;
mod subvol;

use bitflags::bitflags;
use std::{
    ffi::CString,
    num::NonZeroU64,
    os::{raw::c_int, unix::prelude::OsStrExt},
    path::Path,
};

pub use error::Error;
pub use qgroup::QgroupInherit;
pub use subvol::*;
pub const FS_TREE_OBJECTID: u64 = 5;

pub fn subvolume_info<P: AsRef<Path>>(
    path: P,
    id: Option<NonZeroU64>,
) -> Result<SubvolumeInfo, Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let cid = id.map_or(0, |i| i.get());
    let mut out = SubvolumeInfo::new();
    unsafe {
        let errcode = ffi::btrfs_util_subvolume_info(cpath.as_ptr(), cid, out.as_ptr());
        if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
            return Err(errcode.into());
        }
    }
    Ok(out)
}

bitflags! {
    #[derive(Default)]
    pub struct DeleteSubvolumeFlags: c_int {
        const RECURSIVE = ffi::BTRFS_UTIL_DELETE_SUBVOLUME_RECURSIVE as c_int;
    }
}

/// Delete a subvolume or snapshot
pub fn delete_subvolume<P: AsRef<Path>>(path: P, flags: DeleteSubvolumeFlags) -> Result<(), Error> {
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let cflags = flags.bits();
    unsafe {
        let errcode = ffi::btrfs_util_delete_subvolume(cpath.as_ptr(), cflags);
        if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
            return Err(errcode.into());
        }
    }
    Ok(())
}

bitflags! {
    #[derive(Default)]
    pub struct CreateSnapshotFlags: c_int {
        const READ_ONLY	= ffi::BTRFS_UTIL_CREATE_SNAPSHOT_READ_ONLY as c_int;
        const RECURSIVE = ffi::BTRFS_UTIL_CREATE_SNAPSHOT_RECURSIVE as c_int;
    }
}

/// Create a new snapshot from a source subvolume
pub fn create_snapshot<P: AsRef<Path>>(
    source: P,
    path: P,
    flags: CreateSnapshotFlags,
    qgroup: Option<QgroupInherit>,
) -> Result<(), Error> {
    let csource = CString::new(source.as_ref().as_os_str().as_bytes()).unwrap();
    let cpath = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let cflags = flags.bits();
    let unused = std::ptr::null_mut();
    let cqgroup: *mut ffi::btrfs_util_qgroup_inherit = if let Some(qg) = qgroup {
        qg.as_ptr()
    } else {
        std::ptr::null_mut()
    };
    unsafe {
        let errcode = ffi::btrfs_util_create_snapshot(
            csource.as_ptr(),
            cpath.as_ptr(),
            cflags,
            unused,
            cqgroup,
        );
        if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
            return Err(errcode.into());
        }
    }
    Ok(())
}
