extern crate serial;
extern crate rscam;

use rscam::{Camera, Config};
use std::io::{Read,Write};
use std::io;
use std::time::Duration;
use std::thread;
use serial::SerialPort;

mod preprocess;

// Lasers are mounted on either side of the camera. "Left" and "Right"
// here refer to the camera's point of view, not the user's!
#[derive (Copy, Clone)]
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
    ( $x:expr ) => (try!($x.map_err(|err| KerboError::Serial(err))));
}

macro_rules! try_io {
    ( $x:expr ) => (try!($x.map_err(|err| KerboError::Io(err))));
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
        frame.map_err(|err| KerboError::Io(err))
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

fn main() {
    let mut k = Kerbo::new_from_portname("/dev/ttyACM0","/dev/video1").unwrap();
    println!("Flushing port...");
    k.flush_port_input().unwrap();
    println!("Flushed port.");
    k.laser(Side::Left, false).unwrap();
    k.laser(Side::Right, false).unwrap();
    k.scan("test_scan",64);
}
