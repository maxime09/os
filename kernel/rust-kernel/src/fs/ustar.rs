use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use super::vfs::{self, FsDriver, Inode, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType{
    RegularFile,
    Folder,
    Other,
}

#[derive(Debug)]
pub struct Header{
    name: String,
    size: usize,
    file_type: FileType,
    start_addr: usize,
}

impl Header {
    pub fn is_readable(&self) -> bool{
        match self.file_type{
            FileType::RegularFile => true,
            _ => false
        }
    }
}


pub fn parse_octal_size(data: &[u8]) -> usize{
    let mut result = 0;
    for i in 0..12{
        if data[i] == 0{
            return result;
        }
        result *= 8;
        result += (data[i] - b'0') as usize;
    }
    result
}

pub fn parse_name(data: &[u8]) -> String{
    let mut result = String::new();
    let mut i = 0;
    while i < 100 && data[i] != 0{
        result.push(data[i] as char);
        i += 1;
    }
    result
}

pub fn parse_header(data: &[u8], start: usize) -> Option<Header>{
    let ustart_start = start + 257;
    if !is_valid_header(&data[ustart_start..ustart_start+5]){
        return None;
    }
    let size_start = start + 124;
    let size = parse_octal_size(&data[size_start..size_start+12]);
    let file_type = match data[start + 156]{
        b'0'|0 => FileType::RegularFile,
        b'5' => FileType::Folder,
        _ => FileType::Other
    };
    let name = parse_name(&data[start..start+100]);
    let start_addr = start + 512;
    Some(Header { name, size, file_type, start_addr})
}

pub fn is_valid_header(data: &[u8]) -> bool{
    let pattern = b"ustar";
    for i in 0..5{
        if data[i] != pattern[i]{
            return false;
        }
    }
    true
}

pub fn parse_file(data: &[u8]) -> Vec<Header>{
    let mut pos = 0;
    let mut res = Vec::new();
    while let Some(header) = parse_header(data, pos){
        let offset = if header.file_type == FileType::RegularFile && header.size != 0{
            (((header.size + 511) / 512) + 1) * 512
        }else{
            512
        };
        pos += offset;
        res.push(header);
    }
    res
}

pub fn headers_to_fs(headers: Vec<Header>, data: Box<[u8]>) -> Inode{
    let mut driver = UstarDriver::new(data);
    let mut id = 0;
    let mut root = Inode::new_folder(id);
    let root_header = Header{ name: String::new(), size: 0, file_type: FileType::Folder, start_addr: 0};
    driver.insert_header(id, root_header);
    id += 1;
    for header in headers.into_iter(){
        let mut path = PathBuf::from(header.name.as_ref());
        let mut parent = &mut root;
        if path.is_empty(){
            panic!("File with no path");
        }
        while !path.is_basename(){
            let component = path.split_first_component().unwrap();
            parent = parent.search_in_folder_mut(&component).unwrap();
        }

        let name = path.split_first_component().unwrap();

        match header.file_type{
            FileType::RegularFile => {
                let node = Inode::new_file(id);
                driver.insert_header(id, header);
                id += 1;
                parent.add_to_folder(node, name).unwrap();
            },
            FileType::Folder => {
                let node = Inode::new_folder(id);
                driver.insert_header(id, header);
                id += 1;
                parent.add_to_folder(node, name).unwrap();
            }
            _ => {}
        }
    }
    let mount_point = Inode::new_mountpoint(root, driver, 0);
    mount_point
}

pub struct UstarDriver{
    headers: BTreeMap<usize, Header>,
    data: Box<[u8]>
}

impl UstarDriver{
    pub fn new(data: Box<[u8]>) -> Self{
        UstarDriver { headers: BTreeMap::new(), data }
    }

    pub fn insert_header(&mut self, id: usize, header: Header){
        self.headers.insert(id, header);
    }

    pub fn get_header(&self, id: usize) -> Result<&Header, vfs::Error>{
        self.headers.get(&id).ok_or(vfs::Error::NotFound)
    }
}

impl FsDriver for UstarDriver{
    fn get_size(&self, node: &Inode) -> Result<usize, vfs::Error> {
        let id = node.get_id();
        let header = self.get_header(id)?;
        Ok(header.size)
    }
    
    fn read(&self, node: &Inode, pos: usize, requested_amount: usize) -> Result<Box<[u8]>, vfs::Error> {
        let id = node.get_id();
        let header = self.get_header(id)?;
        let size = header.size;
        if header.is_readable(){
            let start = pos.min(size);
            let end = (start + requested_amount).min(size);
            let start_offset = header.start_addr + start;
            let end_offset = header.start_addr + end;
            let data = &self.data[start_offset..end_offset];
            Ok(Box::from(data))
        }else{
            Err(vfs::Error::NotAReadableFile)
        }
    }
}