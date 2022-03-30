pub struct QgroupInherit(*mut ffi::btrfs_util_qgroup_inherit);

impl QgroupInherit {
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
