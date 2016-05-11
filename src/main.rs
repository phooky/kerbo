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

impl Kerbo {
    pub fn new_from_port(mut port : serial::SystemPort) -> serial::Result<Kerbo> {
        // flush serial port. This should move to the serial package
        port.set_timeout(Duration::new(0,0));
        let mut buf = Vec::new();
        port.read_to_end(&mut buf);
        port.set_timeout(Duration::from_millis(500));
        Ok(Kerbo { control_port : port, turntable_position : 0 as u16 })
    }

    pub fn new_from_portname<T: AsRef<OsStr> + ?Sized>(portname: &T) -> serial::Result<Kerbo> {
        serial::open("/dev/ttyACM0").and_then(|mut port| Kerbo::new_from_port(port))
    }
    
    fn wait_for_ok(&mut self, millis : u64) -> serial::Result<()> {
        let mut remainder = millis;
        let step_ms : u64 = 1;
        let timeout = Duration::from_millis(step_ms);
        self.control_port.set_timeout(timeout);
        let mut buf = Vec::new();
        while remainder >= step_ms {
            self.control_port.read_to_end(&mut buf);
            remainder = remainder - step_ms;
        }
        println!("{:?}",String::from_utf8(buf).unwrap());
        Ok(())
    }
    
    pub fn laser(&mut self, side : Side, on : bool) -> serial::Result<()> {
        let cmd = match side {
            Side::Left => "r",
            Side::Right => "l",
        }.to_string() + match on {
            true => "ff",
            false => "00",
        } + "\n";
        self.control_port.write(cmd.as_bytes());
        self.wait_for_ok(5)
    }

    pub fn go_to_position(&mut self, position : u16) -> serial::Result<u16> {
        let offset = position as i32 - self.turntable_position as i32;
        let offset_str = offset.to_string();
        let cmd = if offset < 0 {
            offset_str
        } else {
            "+".to_string() + &offset_str
        } + "\n";
        println!("{:?}",cmd);
        self.control_port.write(cmd.as_bytes());
        self.wait_for_ok(3000);
        self.turntable_position = position;
        Ok(position)
    }
    
}

fn main() {
    let mut k = Kerbo::new_from_portname("/dev/ttyACM0").unwrap();
    k.laser(Side::Left, true).unwrap();
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Left, false).unwrap();
    k.laser(Side::Right, true).unwrap();
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Right, false).unwrap();
    k.go_to_position(100).unwrap();
    k.go_to_position(0).unwrap();
}
