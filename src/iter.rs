pub struct SubvolumeIterator(*mut ffi::btrfs_util_subvolume_iterator);

impl Drop for SubvolumeIterator {
    fn drop(&mut self) {
        unsafe {
            ffi::btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}
