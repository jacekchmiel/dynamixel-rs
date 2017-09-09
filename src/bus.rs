use std::ffi::OsStr;
use std::io::{Read, Write};
use std::{thread, time};
use std;
use std::fmt;

extern crate serial;

use self::serial::prelude::*;

use super::packets;


pub type Result<T> = std::result::Result<T, Error>;

/// Categories of errors that can occur when interacting Dynamixel Bus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    // Error related to serial port internals
    SerialError(serial::ErrorKind),

    // IoError happened during bus read
    ReadError(std::io::ErrorKind),

    // IoError happened during bus write
    WriteError(std::io::ErrorKind),

    // Not all data transferred over serial
    TransferError,

    // Cannot parse response
    DataError(packets::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::SerialError(e) => write!(f, "serial error occured: {:?}", e),
            &Error::ReadError(e) => write!(f, "read failed: {:?}", e),
            &Error::WriteError(e) => write!(f, "write failed: {:?}", e),
            &Error::DataError(e) => write!(f, "cannot parse response: {:?}", e),
            &Error::TransferError => write!(f, "not all data transferred"),
        }
    }
}

pub struct Bus {
    port: serial::SystemPort
}

pub trait HalfDuplex {
    fn exchange(&mut self, p: &packets::Request) -> Result<packets::Status>;
}

impl HalfDuplex for Bus {
    fn exchange(&mut self, p: &packets::Request) -> Result<packets::Status> {
        let request_data = p.serialized();

        match self.port.write(request_data.as_slice()) {
            Err(err) => Err(Error::WriteError(err.kind())),
            Ok(len) if len != request_data.len() => Err(Error::TransferError),
            Ok(_) => {
                match self.read_packet() {
                    Ok(data) => match packets::Status::from_bytes(data.as_slice()) {
                        Ok(s) => Ok(s),
                        Err(e) => {
                            Err(Error::DataError(e))
                        }
                    },
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl Bus {
    pub fn open<T: AsRef<OsStr> + ? Sized>(port: &T, baud: u32) -> Result<Bus> {
        let mut p = serial::open(port).map_err(|e| Error::SerialError(e.kind()))?;

        let mut s = serial::PortSettings::default();
        s = serial::PortSettings { baud_rate: serial::BaudRate::from_speed(baud as usize), ..s };
        p.configure(&s).unwrap();
        Ok(Bus { port: p })
    }

    fn read_packet(&mut self) -> Result<Vec<u8>> {
        let mut output: Vec<u8> = Vec::new();
        let mut local_buf: &mut [u8] = &mut [0; 128];
        loop {
            match self.port.read(local_buf) {
                Ok(size) if size == 0 => {
                    break Err(Error::TransferError);
                },
                Ok(size) => {
                    info!("Read {} bytes", size);
                    output.extend(local_buf[..size].as_ref());
                    if packets::Status::is_constructible_from(output.as_slice()) {
                        info!("Packet complete");
                        break Ok(output);
                    } else {
                        thread::sleep(time::Duration::from_millis(1));
                    }
                },
                Err(e) => {
                    break Err(Error::ReadError(e.kind()));
                }
            }
        }
    }
}


