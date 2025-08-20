// use ql_driver::driver::{
//     PrinterCommand, PrinterCommandMode, PrinterCommander, PrinterStatus, commands,
// };

use ql_driver::{Printer, image::ImageBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut printer = Printer::open("/dev/usb/lp0")?;
    let file = std::env::args().nth(1).expect("expect file argument");

    let image = ImageBuilder::open(&file)?.dither().render();
    printer.print_image(&image)?;

    // let settings = PrinterSettings::QL_500;

    // let job = PrintJob::rasterize_image(image, settings);
    // printer.print(&job)?;

    // println!("Hello, world!");

    // // let printer = driver::Printer::new("/dev/usb/lp0").unwrap();

    // let mut commander = PrinterCommander::main("/dev/usb/lp0")?;

    // commander.send_command(Reset)?;
    // commander.send_command(Initialize)?;

    // let status = commander.send_command_read(StatusInfoRequest)?;
    // commander.send_command(SetCommandMode::new(PrinterCommandMode::EscpNormal))?;
    // commander.send_command(SetPrintInformation::new(status, 1))?;

    // let mut data = [0; 90];
    // data.iter_mut()
    //     .enumerate()
    //     .for_each(|(a, b)| *b = (a % 2 == 0).then_some(0xFF).unwrap_or(0x00));

    // let transfer = RasterGraphicsTransfer::new(&data)?;
    // commander.send_command(transfer)?;
    // commander.send_command(Print)?;

    Ok(())
}
