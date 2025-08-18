pub mod commands;
mod encode;
pub mod types;

use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{
    driver::commands::{Command, CommandResponse},
    error::PrinterError,
};

pub struct Printer {
    buffer: Box<[u8]>,
    fd: std::fs::File,
}

impl Printer {
    const BUF_SIZE: usize = 64;

    pub fn new(path: &str) -> Result<Self, PrinterError> {
        let fd = File::options().read(true).write(true).open(path)?;
        let buffer = Box::new([0u8; Self::BUF_SIZE]);

        Ok(Self { fd, buffer })
    }

    pub fn read(&mut self, length: usize) -> Result<&[u8], PrinterError> {
        assert!(length < Self::BUF_SIZE);

        // try 10 times and return ReadTimeout if none of the attempts are ok
        (0..10)
            .into_iter()
            .map(|_| self.fd.read_exact(&mut self.buffer[..length])) // for each attempt try the read_exact
            .filter_map(Result::ok) // filter out None values
            .next() // next with filter should iterate until an Ok value is found
            .ok_or(PrinterError::ReadTimeout)?; // if none is found return a timeout error

        Ok(&self.buffer[..length])
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), PrinterError> {
        self.fd.write_all(data)?;
        Ok(())
    }
}

pub struct PrinterCommander {
    printer: Printer,
}

impl PrinterCommander {
    pub fn main(path: &str) -> Result<Self, PrinterError> {
        let lp = Printer::new(path)?;

        Ok(Self { printer: lp })
    }

    pub fn send_command<C: Command>(&mut self, command: C) -> Result<(), PrinterError> {
        command.send_command(&mut self.printer)
    }

    pub fn send_command_read<C: CommandResponse>(
        &mut self,
        command: C,
    ) -> Result<C::Response, PrinterError> {
        command.send_command(&mut self.printer)?;
        command.read_response(&mut self.printer)
    }
}
