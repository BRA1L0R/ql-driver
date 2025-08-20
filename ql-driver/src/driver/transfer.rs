use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};

use crate::{
    driver::{
        PrinterLink,
        command::{CommandTransfer, Ship},
    },
    error::QlDriverError,
};

pub struct RasterShip<'a> {
    printer: &'a mut PrinterLink,
    line: bitvec::vec::BitVec<u8, Msb0>,
}

impl RasterShip<'_> {
    const CONTROL: [u8; 3] = [b'g', 0x00, 0x00];
    const CONTROL_BITS: usize = Self::CONTROL.len() * 8;

    fn current_length(&self) -> usize {
        self.line.len() - Self::CONTROL_BITS // account for control bytes at the start
    }

    pub fn push_bits(&mut self, data: &BitSlice) {
        assert!(data.len() + self.current_length() < (u8::MAX as usize * 8));

        self.line.extend_from_bitslice(data);
    }

    pub fn fill_remaining(&mut self, target_size: usize) {
        assert!(target_size >= self.current_length());

        // let remaining = target_size - self.current_length();
        println!("prev size {}", self.line.len());
        self.line.resize(target_size + Self::CONTROL_BITS, false);
        println!("after size {}", self.line.len());
    }

    pub fn send_line(&mut self) -> Result<(), QlDriverError> {
        let bytes = self.line.len().div_ceil(8);

        // fills the remaining unused bits to a known value
        self.line.set_uninitialized(false);
        let data = self.line.as_raw_mut_slice();

        // set the byte transfer length and then write to the printer
        data[2] = bytes.try_into().unwrap();
        self.printer.write(data)?;

        println!("now sending {data:?}");

        // restore the first 3 bytes of control data
        self.line.truncate(Self::CONTROL_BITS);
        Ok(())
    }
}

impl Ship for RasterShip<'_> {
    fn end(self) {}
}

pub struct RasterTransfer;

impl CommandTransfer for RasterTransfer {
    type Ship<'a> = RasterShip<'a>;
    fn start_transfer<'a>(
        &self,
        printer: &'a mut PrinterLink,
    ) -> Result<Self::Ship<'a>, QlDriverError> {
        // these are the raw command bytes that will be sent
        let line: BitVec<u8, Msb0> = BitVec::from_slice(&[b'g', 0x00, 0x00]);

        Ok(RasterShip { printer, line })
    }
}
