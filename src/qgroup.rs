use std::{ptr, slice};

use ffi::size_t;

use crate::Error;

/// qgroup inheritance specifier.
pub struct QgroupInherit(*mut ffi::btrfs_util_qgroup_inherit);

impl QgroupInherit {
    pub fn new() -> Result<Self, Error> {
        let mut ret: *mut ffi::btrfs_util_qgroup_inherit = ptr::null_mut();

        let errcode = unsafe { ffi::btrfs_util_create_qgroup_inherit(0, &mut ret) };
        if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
            Err(errcode.into())
        } else {
            Ok(QgroupInherit(ret))
        }
    }

    /// Adds inheritance from a qgroup to this qgroup inheritance specifier.
    pub fn add_group(&mut self, qgroup_id: u64) -> Result<(), Error> {
        let mut ptr = self.as_ptr();

        let errcode = unsafe { ffi::btrfs_util_qgroup_inherit_add_group(&mut ptr, qgroup_id) };

        if errcode != ffi::btrfs_util_error_BTRFS_UTIL_OK {
            Err(errcode.into())
        } else {
            self.0 = ptr;
            Ok(())
        }
    }

    /// Returns the qgroups this qgroup inheritance specifier contains.
    pub fn groups(&self) -> &[u64] {
        let self_ptr = self.as_ptr();
        let mut ret_ptr: *const u64 = ptr::null();
        let mut ret_size: size_t = 0;
        unsafe {
            ffi::btrfs_util_qgroup_inherit_get_groups(self_ptr, &mut ret_ptr, &mut ret_size);
            slice::from_raw_parts(ret_ptr, ret_size as usize)
        }
    }

    pub fn as_ptr(&self) -> *mut ffi::btrfs_util_qgroup_inherit {
        self.0
    }
}

impl Drop for QgroupInherit {
    fn drop(&mut self) {
        unsafe {
            ffi::btrfs_util_destroy_qgroup_inherit(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::QgroupInherit;

    #[test]
    fn test_new() {
        let inherit = QgroupInherit::new().unwrap();
        assert_eq!(inherit.groups(), []);
    }

    #[test]
    fn test_add_group() {
        let mut inherit = QgroupInherit::new().unwrap();
        inherit.add_group(1).unwrap();
        assert_eq!(inherit.groups(), [1]);
        inherit.add_group(2).unwrap();
        assert_eq!(inherit.groups(), [1, 2]);
        inherit.add_group(3).unwrap();
        assert_eq!(inherit.groups(), [1, 2, 3]);
    }
}
