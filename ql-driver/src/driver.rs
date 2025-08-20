pub mod command;
pub mod commands;
mod encode;
pub mod types;

use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{
    driver::command::{Command, CommandResponse},
    error::QlDriverError,
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

    pub fn new(path: &str) -> Result<Self, QlDriverError> {
        let fd = File::options().read(true).write(true).open(path)?;
        let buffer = Box::new([0u8; Self::BUF_SIZE]);

        Ok(Self { fd, buffer })
    }

    pub fn read(&mut self, length: usize) -> Result<&[u8], QlDriverError> {
        assert!(length < Self::BUF_SIZE);

        // try 10 times and return ReadTimeout if none of the attempts are ok
        (0..10)
            .map(|_| self.fd.read_exact(&mut self.buffer[..length])) // for each attempt try the read_exact
            .filter_map(Result::ok) // filter out None values
            .next() // next with filter should iterate until an Ok value is found
            .ok_or(QlDriverError::ReadTimeout)?; // if none is found return a timeout error

        Ok(&self.buffer[..length])
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), QlDriverError> {
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
    pub fn main(path: &str) -> Result<Self, QlDriverError> {
        let lp = PrinterLink::new(path)?;

        Ok(Self { printer: lp })
    }

    pub fn send_command<C: Command>(&mut self, command: C) -> Result<(), QlDriverError> {
        command.send_command(&mut self.printer)
    }

    pub fn send_command_read<C: CommandResponse>(
        &mut self,
        command: C,
    ) -> Result<C::Response, QlDriverError> {
        command.send_command(&mut self.printer)?;
        command.read_response(&mut self.printer)
    }
}
