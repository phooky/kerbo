#![feature(zero_one)]
#![allow(dead_code,unused_imports)]
extern crate serial;
extern crate docopt;
extern crate kerbo;

use std::io::{Read,Write};
use std::io;
use std::time::Duration;
use std::thread;
use std::error::Error;
use docopt::Docopt;
use regex::Regex;

use kerbo::{Kerbo, KerboError, ImageType, Side};

mod preprocess;
mod img_proc;


const USAGE: &'static str = "
Usage: kerbo [options]

Options:
  -h, --help
  --serial=<port>     Use the specified serial device. [default: /dev/ttyACM0]
  --video=<video>     Use the specified video device.  [default: /dev/video1]
  --scan-data=<path>  Place/use the raw scan files in the given directory. [default: ./scan-data/]
  --skip-scan         Bypass the physical scanning step and use the data provided
                      by the last scan.
";

#[macro_use] extern crate lazy_static;
extern crate regex;

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
                _ => None
            }
        }
    }
}

use std::collections::HashMap;

fn main() {
    let argv = std::env::args();
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv.into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let scan_path = args.get_str("--scan-data");
    if !args.get_bool("--skip-scan") {
        let port_path = args.get_str("--serial");
        let video_path = args.get_str("--video");
        match Kerbo::new_from_portname(port_path,video_path) {
            Ok(mut k) => {
                println!("Flushing port...");
                k.flush_port_input().unwrap();
                println!("Flushed port.");
                k.laser(Side::Left, false).unwrap();
                k.laser(Side::Right, false).unwrap();
                let prefix = scan_path.to_string() + "scan";
                std::fs::create_dir_all(scan_path).unwrap();
                k.scan(prefix.as_str(),64);
            },
            Err(e) => {
                let stderr = &mut std::io::stderr();
                let detail = match e {
                    KerboError::Serial(se) =>
                        format!("Could not open {}: {}",port_path,se.description()),
                    KerboError::Io(ioe) => ioe.description().to_string(),
                    KerboError::Protocol(s) => s.description().to_string(),
                };
                writeln!(stderr, "Couldn't connect to Kerbo: {}. Exiting!",detail).unwrap();
                std::process::exit(1);
            },
        }
    }
    println!("Processing scan dir '{}'...",scan_path);

    struct ImgSet {
        l : Option<String>,
        r : Option<String>,
        n : Option<String>,
        w : Option<String>,
    }
    use std::ops::{IndexMut,Index};
    
    impl Index<ImageType> for ImgSet {
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
    impl IndexMut<ImageType> for ImgSet {
        fn index_mut<'a>(&'a mut self, idx : ImageType) -> &'a mut Option<String> {
            match idx {
                ImageType::None => & mut self.n,
                ImageType::Left => & mut self.l,
                ImageType::Right => & mut self.r,
                ImageType::Raw => & mut self.w,
            }
        }
    }
          
    let mut image_map = HashMap::<u16,ImgSet>::new();
    for p in std::fs::read_dir(scan_path).unwrap() {
        let path = p.unwrap().file_name().to_str().unwrap().to_string();
        match parse_scan_path(path.as_str()) {
            Some( (num, imgType) ) => {
                if !image_map.get(&num).is_some() {
                    let v = ImgSet { l : None, r : None, n : None, w : None };
                    image_map.insert(num, v);
                }
                let mut e = image_map.get_mut(&num).unwrap();
                e[imgType] = Some(path);
            },
            None => {
                println!("Ignoring path {}",path);
            },
        }
    }
    let complete = image_map.iter()
        .filter(|&(_,x)| x.l.is_some() && x.r.is_some() && x.n.is_some())
        .count();
    let incomplete = image_map.len() - complete;
    println!("Found {} complete scan images ({} incomplete).", complete, incomplete);
    println!("Dumping subtractive images");
    fn img_from_path(path : String, rz : (usize, usize)) -> img_proc::MemImage<u8> {
        img_proc::MemImage::from_iterator(
            std::fs::File::open(path).unwrap().bytes().map(|x| x.unwrap()),
            rz)
    }
    for (idx,iset) in image_map {
        let resolution = (1280, 1024);
        let limg = img_from_path(iset.l.unwrap(),resolution);
        let rimg = img_from_path(iset.r.unwrap(),resolution);
        let nimg = img_from_path(iset.n.unwrap(),resolution);
        let ladj = &limg - &nimg;
        let radj = &rimg - &nimg;
    }
}
