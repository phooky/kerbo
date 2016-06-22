#![feature(zero_one)]
#![allow(dead_code,unused_imports)]
extern crate serial;
extern crate docopt;
extern crate kerbo;
#[macro_use] extern crate lazy_static;
extern crate regex;

use std::io::{Read,Write};
use std::io;
use std::time::Duration;
use std::thread;
use std::error::Error;
use docopt::Docopt;

use kerbo::{KerboError, ImageType};
use kerbo::hw::KerboHW;
use kerbo::img::ImgSet;

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

fn main() {
    let argv = std::env::args();
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv.into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let scan_path = args.get_str("--scan-data");
    if !args.get_bool("--skip-scan") {
        let port_path = args.get_str("--serial");
        let video_path = args.get_str("--video");
        match KerboHW::new_from_portname(port_path,video_path) {
            Ok(mut k) => {
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
          
    let mut img_set = ImgSet::new_from_path(scan_path);
    let complete = img_set.map.iter()
        .filter(|&(_,x)| x.is_complete())
        .count();
    let incomplete = img_set.map.len() - complete;
    println!("Found {} complete scan images ({} incomplete).", complete, incomplete);
    println!("Dumping subtractive images");
    /*
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
*/
}
