extern crate serial;

use std::io::Write;
use std::time::Duration;
use std::thread;

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
    pub fn new(port : serial::SystemPort) -> Kerbo {
        Kerbo { control_port : port, turntable_position : 0 }
    }

    fn laser(&mut self, side : Side, on : bool) -> serial::Result<()> {
        let cmd = match (side) {
            Side::Left => "r",
            Side::Right => "l",
        }.to_string() + match(on) {
            true => "ff",
            false => "00",
        } + "\n";
        self.control_port.write(cmd.as_bytes());
        Ok(())
    }
}

fn main() {
    let mut k = Kerbo::new(serial::open("/dev/ttyACM0").unwrap());
    k.laser(Side::Left, true);
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Left, false);
    k.laser(Side::Right, true);
    thread::sleep(Duration::from_millis(500));
    k.laser(Side::Right, false);
}
