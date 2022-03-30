mod common;

use common::setup;
use libbtrfsutil::subvolume_info;
use std::{
    num::NonZeroU64,
    process::Command,
    time::{Duration, SystemTime},
};

use crate::common::CommandExt;

#[test]
fn test_subvolume_info() {
    let device = setup();
    let subvol_path = device.mountpoint().unwrap().clone().join("subvol");
    Command::new("sudo")
        .args(["btrfs", "subvolume", "create"])
        .arg(&subvol_path)
        .call()
        .unwrap();

    let snapshot_path = device.mountpoint().unwrap().clone().join("snapshot");
    Command::new("sudo")
        .args(["btrfs", "subvolume", "snapshot"])
        .arg(&subvol_path)
        .arg(&snapshot_path)
        .call()
        .unwrap();

    let root_info = subvolume_info(device.mountpoint().unwrap(), None).unwrap();
    assert_eq!(root_info.id(), 5);
    assert_eq!(root_info.parent_id(), None);
    assert_eq!(root_info.dir_id(), None);
    assert_eq!(root_info.flags(), 0);
    assert!(!root_info.uuid().is_nil());
    assert_eq!(root_info.parent_uuid(), None);
    assert_eq!(root_info.received_uuid(), None);
    assert_ne!(root_info.generation(), 0);
    assert_ne!(root_info.ctransid(), 0);
    assert_eq!(root_info.otransid(), 0);
    assert_eq!(root_info.stransid(), None);
    assert_eq!(root_info.rtransid(), None);
    assert_ne!(root_info.ctime(), 0);
    assert_ne!(root_info.otime(), 0);
    assert_eq!(root_info.stime(), None);
    assert_eq!(root_info.stime_nsec(), None);
    assert_eq!(root_info.rtime(), None);
    assert_eq!(root_info.rtime_nsec(), None);
    assert_eq!(
        root_info
            .created()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        Duration::new(root_info.otime() as u64, root_info.otime_nsec() as u32)
    );
    assert_eq!(
        root_info
            .changed()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        Duration::new(root_info.ctime() as u64, root_info.ctime_nsec() as u32)
    );
    assert_eq!(root_info.received(), None);

    let subvol_info = subvolume_info(subvol_path, None).unwrap();
    assert_eq!(subvol_info.id(), 256);
    assert_eq!(subvol_info.parent_id(), NonZeroU64::new(5));
    assert_eq!(subvol_info.dir_id(), NonZeroU64::new(256));
    assert_eq!(subvol_info.flags(), 0);
    assert!(!subvol_info.uuid().is_nil());
    assert_eq!(subvol_info.parent_uuid(), None);
    assert_eq!(subvol_info.received_uuid(), None);
    assert_ne!(subvol_info.generation(), 0);
    assert_ne!(subvol_info.ctransid(), 0);
    assert!(subvol_info.otransid() > root_info.otransid());
    assert_eq!(subvol_info.stransid(), None);
    assert_eq!(subvol_info.rtransid(), None);
    assert_ne!(subvol_info.ctime(), 0);
    assert_ne!(subvol_info.otime(), 0);
    assert_eq!(subvol_info.stime(), None);
    assert_eq!(subvol_info.stime_nsec(), None);
    assert_eq!(subvol_info.rtime(), None);
    assert_eq!(subvol_info.rtime_nsec(), None);
    assert_eq!(
        subvol_info
            .created()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        Duration::new(subvol_info.otime() as u64, subvol_info.otime_nsec() as u32)
    );
    assert_eq!(
        subvol_info
            .changed()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        Duration::new(subvol_info.ctime() as u64, subvol_info.ctime_nsec() as u32)
    );
    assert_eq!(subvol_info.received(), None);

    let snapshot_info = subvolume_info(snapshot_path, None).unwrap();
    assert_eq!(snapshot_info.parent_id(), NonZeroU64::new(5));
    assert_eq!(snapshot_info.dir_id(), NonZeroU64::new(256));
    assert_eq!(snapshot_info.parent_uuid(), Some(subvol_info.uuid()));
}
