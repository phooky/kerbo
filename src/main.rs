extern crate serial;

use std::io::Write;
use std::io::Read;
use std::time::Duration;
use std::thread;
use serial::SerialPort;
use std::ffi::OsStr;

// Lasers are mounted on either side of the camera. "Left" and "Right"
// here refer to the camera's point of view, not the user's!
enum Side {
    Left,
    Right,
}

struct Kerbo {
    control_port : serial::SystemPort,
    turntable_position : u16,
}

/// KerboError is an error enumeration which encompasses both
/// serial port errors emitted by the serial crate, and protocol
/// errors emitted by the Kerbo itself.
#[derive (Debug)]
enum KerboError {
    Serial(serial::Error),
    Io(std::io::Error),
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

    pub fn new_from_port(port : serial::SystemPort) -> Result<Kerbo> {
        Ok(Kerbo { control_port : port, turntable_position : 0 as u16 })
    }

    pub fn new_from_portname<T: AsRef<OsStr> + ?Sized>(portname: &T) -> Result<Kerbo> {
        let port = try_serial!(serial::open(portname));
        Kerbo::new_from_port(port)
    }

    /// Flush serial port of any buffered input. This should ideally be implemented
    /// in the serial crate.
    pub fn flush_port_input(&mut self) -> Result<()> {
        try_serial!(self.control_port.set_timeout(Duration::new(0,0)));
        let mut buf = Vec::new();
        try_io!(self.control_port.read_to_end(&mut buf)); // discarded
        Ok(())
    }

    /// Wait for an OK message (newline terminated), or error.
    fn wait_for_ok(&mut self, timeout_ms : u64) -> Result<()> {
        let mut remainder = timeout_ms;
        let step_ms : u64 = 1; // ms per polling read
        let timeout = Duration::from_millis(step_ms);
        try_serial!(self.control_port.set_timeout(timeout));
        let mut buf = Vec::new();
        while remainder >= step_ms {
            try_io!(self.control_port.read_to_end(&mut buf));
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
        let offset_str = offset.to_string();
        let cmd = if offset < 0 {
            offset_str
        } else {
            "+".to_string() + &offset_str
        } + "\n";
        println!("{:?}",cmd);
        try_io!(self.control_port.write(cmd.as_bytes()));
        try!(self.wait_for_ok(3000));
        self.turntable_position = position;
        Ok(position)
    }
    
}

fn main() {
    let mut k = Kerbo::new_from_portname("/dev/ttyACM0").unwrap();
    k.flush_port_input().unwrap();
    k.laser(Side::Left, true).unwrap();
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Left, false).unwrap();
    k.laser(Side::Right, true).unwrap();
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Right, false).unwrap();
    println!("Go to position 1000");
    k.go_to_position(100).unwrap();
    println!("Go to position 0");
    k.go_to_position(0).unwrap();
}
