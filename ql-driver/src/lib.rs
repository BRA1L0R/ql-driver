pub mod driver;
pub mod error;

pub mod prelude {
    pub use crate::driver::PrinterCommander;
    pub use crate::driver::commands::*;
    pub use crate::driver::types::*;
}
