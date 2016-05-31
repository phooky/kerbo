extern crate serial;
extern crate rscam;
extern crate docopt;

use rscam::{Camera, Config};
use std::io::{Read,Write};
use std::io;
use std::time::Duration;
use std::thread;
use serial::SerialPort;
use docopt::Docopt;
use regex::Regex;

mod preprocess;
mod img_proc;


const USAGE: &'static str = "
Usage: kerbo scan [options]

Options:
  -h, --help
  --serial=<port>     Use the specified serial device. [default: /dev/ttyACM0]
  --video=<video>     Use the specified video device.  [default: /dev/video1]
  --scan-data=<path>  Bypass hardware and use the scan files in the given directory.
";

// Lasers are mounted on either side of the camera. "Left" and "Right"
// here refer to the camera's point of view, not the user's!
#[derive (Copy, Clone, Debug)]
enum Side {
    Left,
    Right,
}

struct Kerbo {
    control_port : serial::SystemPort,
    turntable_position : u16,
    camera_path : String,
}

/// KerboError is an error enumeration which encompasses both
/// serial port errors emitted by the serial crate, and protocol
/// errors emitted by the Kerbo itself.
#[derive (Debug)]
enum KerboError {
    Serial(serial::Error),
    Io(io::Error),
    Protocol(String),
}

/// kerbo results default to KerboError as their error type
type Result<T> = std::result::Result<T,KerboError>;


macro_rules! try_serial {
    ( $x:expr ) => (try!($x.map_err(KerboError::Serial)));
}

macro_rules! try_io {
    ( $x:expr ) => (try!($x.map_err(KerboError::Io)));
}

impl Kerbo {

    pub fn new_from_port
        (port : serial::SystemPort, cam_path : &str) -> Result<Kerbo> {
        Ok(Kerbo { control_port : port,
                  turntable_position : 0 as u16,
                  camera_path : cam_path.to_string() })
    }

    pub fn new_from_portname
        (portname: &str, cam_path : &str) -> Result<Kerbo> {
        let port = try_serial!(serial::open(portname));
        Kerbo::new_from_port(port, cam_path)
    }

    pub fn capture_frame(&mut self) -> Result<rscam::Frame> {
        let mut cam = Camera::new(self.camera_path.as_str()).unwrap();
        cam.start(&Config {
            interval: (2,15), // 7.5fps
            resolution: (1280, 1024),
            format: b"YUYV",
            .. Default::default() }).unwrap();
        let frame = cam.capture();
        cam.stop().unwrap();
        frame.map_err(KerboError::Io)
    }
    
    fn non_blocking_read(&mut self, buf : &mut Vec<u8>) -> Result<()> {
        match self.control_port.read_to_end(buf) {
            Ok(_) => Err(KerboError::Io(io::Error::new(io::ErrorKind::UnexpectedEof,"Port closed!"))),
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => Ok(()),
                _ => Err(KerboError::Io(e)),
            }
        }        
    }

    /// Flush serial port of any buffered input. This should ideally be implemented
    /// in the serial crate.
    pub fn flush_port_input(&mut self) -> Result<()> {
        try_serial!(self.control_port.set_timeout(Duration::new(0,0)));
        let mut buf = Vec::new();
        self.non_blocking_read(&mut buf)
    }

    /// Wait for an OK message (newline terminated), or error.
    fn wait_for_ok(&mut self, timeout_ms : u64) -> Result<()> {
        let mut remainder = timeout_ms;
        let step_ms : u64 = 1; // ms per polling read
        let timeout = Duration::from_millis(step_ms);
        try_serial!(self.control_port.set_timeout(timeout));
        let mut buf = Vec::new();
        while remainder >= step_ms {
            try!(self.non_blocking_read(&mut buf));
            // the clone call below is clumsy. Ask some rustaceans how to get around it.
            if let Some(c) = buf.last() {
                if *c == (b'\n' as u8) {
                    if buf == b"OK\n" {
                        return Ok(())
                    } else {
                        let rstr = String::from_utf8(buf.clone()).unwrap_or(String::from("Incomprehensible string"));
                        return Err(KerboError::Protocol(rstr));
                    }
                }
            }
            remainder = remainder - step_ms;
        }
        println!("{:?}",String::from_utf8(buf).unwrap());
        Err(KerboError::Protocol(String::from("Timeout")))
    }
    
    pub fn laser(&mut self, side : Side, on : bool) -> Result<()> {
        let cmd = match side {
            Side::Left => "r",
            Side::Right => "l",
        }.to_string() + match on {
            true => "ff",
            false => "00",
        } + "\n";
        try_io!(self.control_port.write(cmd.as_bytes()));
        self.wait_for_ok(5)
    }

    pub fn go_to_position(&mut self, position : u16) -> Result<u16> {
        let offset = position as i32 - self.turntable_position as i32;
        if offset == 0 { return Ok(position); }
        let cmd = format!("{:+x}\n",offset);
        println!("{:?}",cmd);
        try_io!(self.control_port.write(cmd.as_bytes()));
        try!(self.wait_for_ok( (offset.abs() as u64 * 10) + 10));
        self.turntable_position = position;
        Ok(position)
    }

    pub fn scan_at(&mut self, position : u16, file_root : &str, side : Option<Side>) {
        self.go_to_position(position).unwrap();
        match side { Some(x) => self.laser(x, true).unwrap(), None => () }
        thread::sleep(Duration::from_millis(50));
        let frame = self.capture_frame().unwrap();
        let path = format!("{}{:4x}{}.yuv",
                           file_root.to_string(),
                           position,
                           match side {
                               None => "N",
                               Some(Side::Left) => "L",
                               Some(Side::Right) => "R", });
        let mut file = std::fs::File::create(path).unwrap();
        match side { Some(x) => self.laser(x, false).unwrap(), None => () }
        file.write_all(&frame[..]).unwrap();
    }

    pub fn scan(&mut self, file_root : &str, increment : u16) {
        let mut pos = 0;
        while pos < 0x1900 {
            println!("scan at {:4x}",pos);
            self.scan_at(pos, file_root, None);
            self.scan_at(pos, file_root, Some(Side::Left));
            self.scan_at(pos, file_root, Some(Side::Right));
            pos = pos + increment;
        }
    }
}

#[macro_use] extern crate lazy_static;
extern crate regex;

fn parse_scan_path(path : &str) -> Option<(u16, Option<Side>)> {
    lazy_static! {
        static ref RE : Regex = Regex::new(r"([a-f0-9]{4})([NLR])\.yuv$").unwrap();
    }
    match RE.captures(path) {
        None => None,
        Some(caps) => {
            let num = u16::from_str_radix(caps.at(1).unwrap(),16).unwrap();
            let sidestr = caps.at(2).unwrap();
            match sidestr {
                "L" => Some( (num,Some(Side::Left)) ),
                "R" => Some( (num,Some(Side::Right)) ),
                "N" => Some( (num,None) ),
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
    if args.get_bool("scan") {
        let port_path = args.get_str("<port>");
        let video_path = args.get_str("<video>");
        let mut k = Kerbo::new_from_portname(port_path,video_path).unwrap();
        println!("Flushing port...");
        k.flush_port_input().unwrap();
        println!("Flushed port.");
        k.laser(Side::Left, false).unwrap();
        k.laser(Side::Right, false).unwrap();
        k.scan("test_scan",64);
    } else if args.get_bool("process") {
        // process existing scans
        let path = args.get_str("<scan-dir>");
        struct ImgSet {
            l : Option<String>,
            r : Option<String>,
            n : Option<String>,
        };
        /*
        let mut image_map = HashMap::<u16,ImgSet>::new();
        let mut contents = std::fs::read_dir(path).unwrap()
            .map(|x| x.unwrap().path().to_str().unwrap());
        for p in contents {
            match parse_scan_path(p) {
                Some( (num, side) ) => {
                    println!("{} {:?}",num,side);
                    let mut e = match image_map.get_mut(&num) {
                        None => { let mut v = ImgSet { l : None, r : None, n : None };
                                  image_map.insert(num, v);
                                  image_map.get_mut(&num).unwrap() },
                        Some(v) => v };
                    match side {
                        None => e.n = Some(p.to_string()),
                        Some(Side::Left) => e.l = Some(p.to_string()),
                        Some(Side::Right) => e.r = Some(p.to_string()),
                    }
                },
                None => (),
            }
        }
*/
        /*
        contents.sort();
        let mut paths = contents.as_slice();
        while paths.len() >= 3 {
            let (l, n, r) = (paths[0].to_str().unwrap(),
                             paths[1].to_str().unwrap(),
                             paths[2].to_str().unwrap());
            paths = &paths[3..];
            println!("process {} {} {} {:?}",l,n,r,parse_scan_path(l));
        }
*/
    }
}
