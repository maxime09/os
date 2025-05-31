use core::fmt::Debug;

use alloc::{boxed::Box, collections::VecDeque, string::String};
use hashbrown::HashMap;


#[derive(Debug)]
pub enum InodeType{
    RegularFile,
    Folder(HashMap<String, Inode>),
    MountPoint(Box<MountPoint>),
}

#[derive(Debug)]
pub struct Inode{
    node_type: InodeType,
    id: usize, // only used by driver
}

impl Inode{
    pub fn new_folder(local_id: usize) -> Self{
        let content = HashMap::new();
        Inode { node_type: InodeType::Folder(content), id: local_id}
    }

    pub fn new_file(local_id: usize) -> Self{
        Inode { node_type: InodeType::RegularFile, id: local_id}
    }

    pub fn new_mountpoint<T: FsDriver + 'static>(root: Inode, driver: T, id: usize) -> Self{
        let mount_point = MountPoint::new(root, driver);
        Inode { node_type: InodeType::MountPoint(Box::new(mount_point)), id }
    }

    pub fn search_in_folder(&self, name: &str) -> Result<&Inode, Error>{
        match &self.node_type{
            InodeType::Folder(content) => {
                content.get(name).ok_or(Error::NotFound)
            },
            InodeType::MountPoint(mountpoint) => {
                mountpoint.root.search_in_folder(name)
            }
            _ => Err(Error::NotAFolder)
        }
    }

    pub fn search_in_folder_mut(&mut self, name: &str) -> Result<&mut Inode, Error>{
        match &mut self.node_type{
            InodeType::Folder(content) => {
                content.get_mut(name).ok_or(Error::NotFound)
            },
            InodeType::MountPoint(mountpoint) => {
                mountpoint.root.search_in_folder_mut(name)
            }
            _ => Err(Error::NotAFolder)
        }
    }

    pub fn search_in_mountpoint(&self, name: &str) -> Result<&Inode, Error>{
        match &self.node_type{
            
            _ => Err(Error::NotAMountpoint)
        }
    }

    pub fn add_to_folder(&mut self, file: Inode, name: String) -> Result<(), Error>{
        match &mut self.node_type{
            InodeType::Folder(content) => {
                content.try_insert(name, file).map(|_| {}).map_err(|_| { Error::FileAlreadyExist })
            },
            _ => Err(Error::NotAFolder)
        }
    }

    pub fn get_id(&self) -> usize{
        self.id
    }

    pub fn get_mountpoint(&self) -> Result<&MountPoint, Error>{
        match &self.node_type{
            InodeType::MountPoint(mountpoint) => {
                Ok(mountpoint)
            }
            _ => Err(Error::NotAMountpoint)
        }
    }

    pub fn get_mountpoint_mut(&mut self) -> Result<&mut MountPoint, Error>{
        match &mut self.node_type{
            InodeType::MountPoint(mountpoint) => {
                Ok(mountpoint)
            }
            _ => Err(Error::NotAMountpoint)
        }
    }

    pub fn find(&self, mut path: PathBuf) -> Result<&Inode, Error>{
        if let Some(component) = path.split_first_component(){
            let next = self.search_in_folder(&component)?;
            next.find(path)
        }else {
            Ok(self)
        }
    }

    pub fn get_size(&self, mountpoint: &MountPoint) -> Result<usize, Error>{
        mountpoint.driver.get_size(&self)
    }

    pub fn read(&self, mountpoint: &MountPoint, node: &Inode, pos: usize, requested_amount: usize) -> Result<Box<[u8]>, Error>{
        mountpoint.driver.read(node, pos, requested_amount)
    }
}

#[derive(Debug)]
pub enum Error{
    NotFound,
    FileAlreadyExist,
    NotAFolder,
    NotAMountpoint,
    NotAReadableFile,
}

#[derive(Debug, Clone)]
pub struct PathBuf{
    components: VecDeque<String>
}

impl PathBuf{
    pub fn is_empty(&self) -> bool{
        self.components.is_empty()
    }

    pub fn split_first_component(&mut self) -> Option<String>{
        self.components.pop_front()
    }

    pub fn is_basename(&self) -> bool{
        self.components.len() == 1
    }
}

impl From<&str> for PathBuf{
    fn from(value: &str) -> Self {
        let mut components = VecDeque::new();
        let mut current_components = String::new();
        for c in value.chars(){
            if c == '/' && !current_components.is_empty(){
                components.push_back(current_components);
                current_components = String::new();
            }else{
                current_components.push(c);
            }
        }
        if !current_components.is_empty(){
            components.push_back(current_components);
        }
        PathBuf { components }
    }
}


pub struct MountPoint{
    root: Inode,
    driver: Box<dyn FsDriver>
}

impl MountPoint{
    pub fn new<T: FsDriver + 'static>(root: Inode, driver: T) -> Self{
        MountPoint { root, driver: Box::new(driver) }
    }
}

impl Debug for MountPoint{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MountPoint").field("root", &self.root).finish_non_exhaustive()
    }
}

pub trait FsDriver {
    fn get_size(&self, node: &Inode) -> Result<usize, Error>;
    fn read(&self, node: &Inode, pos: usize, requested_amount: usize) -> Result<Box<[u8]>, Error>;
}