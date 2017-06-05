use super::ImageType;
use std;
use lazy_static;

pub struct ImgSetEntry {
    l : Option<String>,
    r : Option<String>,
    n : Option<String>,
    w : Option<String>,
}

use std::ops::{IndexMut,Index};

impl Index<ImageType> for ImgSetEntry {
    type Output = Option<String>;
    fn index<'a>(&'a self, idx : ImageType) -> &'a Option<String> {
        match idx {
            ImageType::None => & self.n,
            ImageType::Left => & self.l,
            ImageType::Right => & self.r,
            ImageType::Raw => & self.w,
        }
    }
}

impl IndexMut<ImageType> for ImgSetEntry {
    fn index_mut<'a>(&'a mut self, idx : ImageType) -> &'a mut Option<String> {
        match idx {
            ImageType::None => & mut self.n,
            ImageType::Left => & mut self.l,
            ImageType::Right => & mut self.r,
            ImageType::Raw => & mut self.w,
        }
    }
}

impl ImgSetEntry {
    /// Complete means having left, right, and background scans; "raw" is
    /// not required for "completeness"
    pub fn is_complete(&self) -> bool {
        self.l.is_some() &&
        self.r.is_some() &&
        self.n.is_some()
    }
    pub fn new() -> ImgSetEntry {
        ImgSetEntry { l : None, r : None, n : None, w : None }
    }
}

use regex::Regex;

fn parse_scan_path(path : &str) -> Option<(u16, ImageType)> {
    lazy_static! {
        static ref RE : Regex = Regex::new(r"([a-f0-9]{4})([NLR])\.yuv$").unwrap();
    }
    match RE.captures(path) {
        None => None,
        Some(caps) => {
            let num = u16::from_str_radix(caps.at(1).unwrap(),16).unwrap();
            let sidestr = caps.at(2).unwrap();
            match sidestr {
                "L" => Some( (num,ImageType::Left) ),
                "R" => Some( (num,ImageType::Right) ),
                "N" => Some( (num,ImageType::None) ),
                "W" => Some( (num,ImageType::Raw) ),
                _ => None
            }
        }
    }
}

use std::collections::HashMap;

pub struct ImgSet {
    pub map : HashMap<u16, ImgSetEntry>
}

use std::fs::read_dir;

impl ImgSet {
    pub fn new_from_path(scan_path : &str) -> Result<ImgSet,std::io::Error> {
        let mut img_set = ImgSet { map : HashMap::<u16,ImgSetEntry>::new() };
        let paths = try!(read_dir(scan_path));
        for p in paths {
            let path = p.unwrap().file_name().to_str().unwrap().to_string();
            match parse_scan_path(path.as_str()) {
                Some( (num, img_type) ) => {
                    if !img_set.map.get(&num).is_some() {
                        let v = ImgSetEntry::new();
                        img_set.map.insert(num, v);
                    }
                    let mut e = img_set.map.get_mut(&num).unwrap();
                    e[img_type] = Some(path);
                },
                None => {
                    println!("Ignoring path {}",path);
                },
            }
        }
        Ok(img_set)
    }

}
