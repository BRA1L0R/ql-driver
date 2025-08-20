use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};

use crate::{
    driver::{
        PrinterLink,
        command::{CommandTransfer, Ship},
    },
    error::QlDriverError,
};

pub struct RasterShip<'a> {
    bytes_per_line: u8,

    printer: &'a mut PrinterLink,
    line: bitvec::vec::BitVec<u8, Msb0>,
}

impl RasterShip<'_> {
    const CONTROL: [u8; 3] = [b'g', 0x00, 0x00];

    pub fn push_line(&mut self, data: &BitSlice) {
        let bits_per_line = self.bytes_per_line as usize * 8;
        assert!(data.len() <= bits_per_line);

        let mut control = Self::CONTROL;
        control[2] = self.bytes_per_line;

        self.line.set_uninitialized(false);
        self.line.extend_from_raw_slice(&control);
        self.line.extend_from_bitslice(data);

        let fill_diff = bits_per_line - data.len();
        self.line.resize(self.line.len() + fill_diff, false);
    }
}

impl Ship for RasterShip<'_> {
    fn send(mut self) -> Result<(), QlDriverError> {
        self.line.set_uninitialized(false);
        let data = self.line.as_raw_slice();

        self.printer.write(data)?;

        Ok(())
    }
}

pub struct RasterTransfer {
    bytes_per_line: u8,
}

impl RasterTransfer {
    pub fn new(bytes_per_line: u8) -> Self {
        Self { bytes_per_line }
    }
}

impl CommandTransfer for RasterTransfer {
    type Ship<'a> = RasterShip<'a>;
    fn start_transfer<'a>(
        &self,
        printer: &'a mut PrinterLink,
    ) -> Result<Self::Ship<'a>, QlDriverError> {
        // these are the raw command bytes that will be sent
        let line: BitVec<u8, Msb0> = BitVec::new();

        Ok(RasterShip {
            printer,
            line,
            bytes_per_line: self.bytes_per_line,
        })
    }
}
