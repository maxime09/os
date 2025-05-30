use alloc::{string::String, vec::Vec};

#[derive(Debug, PartialEq, Eq)]
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