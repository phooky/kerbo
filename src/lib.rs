// Top level lib
extern crate serial;
extern crate rscam;

// Hardware Kerbo module
pub mod hw;

use std::io::{Read,Write};
use std::io;
use std::error::Error;
use std::fmt;

/// Indicates that a transaction with the Kerbo hardware has
/// failed due to a protocol-level error.
#[derive (Debug)]
pub struct ProtocolError {
    description : String,
}

impl Error for ProtocolError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

/// A composite error enumeration which encompasses
/// the various ways in which a transaction with the
/// Kerbo hardware could fail: a serial connection error,
/// an IO error when connecting, or a protocol error that
/// could arise from the Kerbo being in a bad state (or
/// the device not responding as expected because it's
/// not the Kerbo at all).
#[derive (Debug)]
pub enum KerboError {
    Serial(serial::Error),
    Io(io::Error),
    Protocol(ProtocolError),
}

impl Error for KerboError {
    fn description(&self) -> &str {
        self.cause().unwrap().description()
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            KerboError::Serial(ref e) => Some(e),
            KerboError::Io(ref e) => Some(e),
            KerboError::Protocol(ref e) => Some(e),
        }
    }
}

impl fmt::Display for KerboError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KerboError::Serial(ref e) => write!(f,"Serial port error: {}",e.description()),
            KerboError::Io(ref e) => write!(f,"I/O error: {}",e.description()),
            KerboError::Protocol(ref e) => write!(f,"Protocol error: {}",e.description()),
        }
    }
}

impl From<serial::Error> for KerboError {
    fn from(err : serial::Error) -> KerboError { KerboError::Serial(err) }
}

impl From<io::Error> for KerboError {
    fn from(err : io::Error) -> KerboError { KerboError::Io(err) }
}

/// Strings are assumed to represent protocol errors.
impl From<String> for KerboError {
    fn from(err : String) -> KerboError { KerboError::Protocol(
        ProtocolError{ description : err } ) }
}

/// The raw scan captures four types of image: with the left laser on, with
/// the right laser on, with neither laser on, and (optionally) a raw image
/// with no filter.
pub enum ImageType {
    /// Left laser on, filter on
    Left,  
    /// Right laser on, filter on
    Right,
    /// Both lasers off, filter on
    None,
    /// Both lasers off, filter off
    Raw,
}

/// Lasers are mounted on either side of the camera. "Left" and "Right"
/// here refer to the camera's point of view, not the user's!
#[derive (Copy, Clone, Debug)]
pub enum Side {
    Left,
    Right,
}


pub trait Kerbo {
    fn capture_frame(&mut self, image_type : ImageType) -> Result<rscam::Frame>;
    fn go_to_position(&mut self, position : u16) -> Result<u16>;
}


/// kerbo results default to KerboError as their error type
pub type Result<T> = std::result::Result<T,KerboError>;
