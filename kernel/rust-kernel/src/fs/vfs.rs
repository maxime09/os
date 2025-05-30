use alloc::{string::String, vec::Vec};

pub enum Inode{
    RegularFile(String),
    Folder(Vec<Inode>),
    MountPoint,
}