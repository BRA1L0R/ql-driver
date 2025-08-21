/// low level commands that can be serialized to the printer
pub mod command;
/// defines traits and methods to serialize data structures in a printer readable format
pub mod encode;
// pub mod transfer;

pub mod types;

use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{
    driver::command::{Command, CommandResponse},
    error::QlError,
    prelude::CommandTransfer,
};

/// Low level serial transport with the Printer
///
/// - Writes are not buffered and directly flushed into the fd
/// - Reads are stored in an internal buffer and a slice to it is given back
pub struct PrinterLink {
    buffer: Box<[u8]>,
    fd: std::fs::File,
}

impl PrinterLink {
    const BUF_SIZE: usize = 64;

    pub fn new(path: &str) -> Result<Self, QlError> {
        let fd = File::options().read(true).write(true).open(path)?;
        let buffer = Box::new([0u8; Self::BUF_SIZE]);

        Ok(Self { fd, buffer })
    }

    pub fn read(&mut self, length: usize) -> Result<&[u8], QlError> {
        assert!(length < Self::BUF_SIZE);

        // try 10 times and return ReadTimeout if none of the attempts are ok
        (0..10)
            .map(|_| self.fd.read_exact(&mut self.buffer[..length])) // for each attempt try the read_exact
            .filter_map(Result::ok) // filter out None values
            .next() // next with filter should iterate until an Ok value is found
            .ok_or(QlError::ReadTimeout)?; // if none is found return a timeout error

        Ok(&self.buffer[..length])
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), QlError> {
        self.fd.write_all(data)?;
        Ok(())
    }
}

/// Command-level interface with the printer.
/// Checkout [`Command`] and [`CommandResponse`]
pub struct PrinterCommander {
    printer: PrinterLink,
}

impl PrinterCommander {
    pub fn main(path: &str) -> Result<Self, QlError> {
        let lp = PrinterLink::new(path)?;

        Ok(Self { printer: lp })
    }

    pub fn send_command<C: Command>(&mut self, command: C) -> Result<(), QlError> {
        command.send_command(&mut self.printer)
    }

    pub fn send_command_read<C: CommandResponse>(
        &mut self,
        command: C,
    ) -> Result<C::Response, QlError> {
        command.send_command(&mut self.printer)?;
        command.read_response(&mut self.printer)
    }

    pub fn send_transfer<'a, T: CommandTransfer>(
        &'a mut self,
        transfer: T,
    ) -> Result<T::Ship<'a>, QlError> {
        transfer.start_transfer(&mut self.printer)
    }
}
