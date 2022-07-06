# libbtrfsutil-rs

[![Crates.io](https://img.shields.io/crates/v/libbtrfsutil)](https://crates.io/crates/libbtrfsutil)
[![docs.rs](https://img.shields.io/docsrs/btrfsutil)](https://docs.rs/libbtrfsutil)

Rust bindings for [libbtrfsutil](https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil).

## Requirements

[libbtrfsutil-sys](libbtrfsutil-sys) generates the bindings at build time through [bindgen](https://github.com/rust-lang/rust-bindgen). As such, you need to have `libclang` and `libbtrfsutil` installed on your system.
To install them on Fedora:

```
# dnf install clang-devel "pkgconfig(libbtrfsutil)"
```
