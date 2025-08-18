// use ql_driver::driver::{
//     PrinterCommand, PrinterCommandMode, PrinterCommander, PrinterStatus, commands,
// };

use ql_driver::driver::{PrinterCommander, commands, types::PrinterCommandMode};

fn main() {
    println!("Hello, world!");

    // let printer = driver::Printer::new("/dev/usb/lp0").unwrap();

    let mut commander = PrinterCommander::main("/dev/usb/lp0").unwrap();

    commander.send_command(commands::Reset).unwrap();
    commander.send_command(commands::Initialize).unwrap();

    let status = commander
        .send_command_read(commands::StatusInfoRequest)
        .unwrap();

    commander
        .send_command(commands::SetCommandMode::new(
            PrinterCommandMode::EscpNormal,
        ))
        .unwrap();

    commander
        .send_command(commands::SetPrintInformation::new(status, 1))
        .unwrap();

    let mut data = [0; 90];
    data.iter_mut()
        .enumerate()
        .for_each(|(a, b)| *b = (a % 2 == 0).then_some(0xFF).unwrap_or(0x00));

    let transfer = commands::RasterGraphicsTransfer::new(&data).unwrap();
    commander.send_command(transfer).unwrap();
    commander.send_command(commands::Print).unwrap();
}
