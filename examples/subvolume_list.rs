use libbtrfsutil::IterateSubvolume;

fn main() {
    for (path, info) in IterateSubvolume::new("/")
        .iter_with_info()
        .unwrap()
        .filter_map(|s| s.ok())
    {
        println!(
            "ID {} gen {} top level {} path {}",
            info.id(),
            info.generation(),
            info.parent_id().unwrap(),
            path.display()
        );
    }
}
