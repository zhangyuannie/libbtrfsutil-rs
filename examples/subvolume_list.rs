use libbtrfsutil::{SubvolumeInfoIterator, SubvolumeIteratorFlags};

fn main() {
    let iter = SubvolumeInfoIterator::new("/", None, SubvolumeIteratorFlags::default()).unwrap();
    for subvol in iter {
        let (path, info) = subvol.unwrap();
        println!(
            "ID {} gen {} top level {} path {}",
            info.id(),
            info.generation(),
            info.parent_id().unwrap(),
            path.display()
        );
    }
}
