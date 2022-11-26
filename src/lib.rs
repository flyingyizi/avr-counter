// 告诉 rustc 只有在禁用 test 标志时才编译 “no-std”
#![cfg_attr(not(test), no_std)]
// 告诉 rustc 只有在启用 test 标志时才编译 “test feature”
#![cfg_attr(test, feature(test))]

// pub enum Error {
//     Bounds,
//     Others,
// }

mod counter;
pub use avr_device;
pub use counter::*;
pub use embedded_hal;
pub use fugit;
pub use nb;
pub use void;

pub mod prelude {
    pub use embedded_hal::timer::CountDown as _;
}
pub use counter::to_prescale_ticks;

// #[cfg(test)]
// pub mod testutil;

#[cfg(feature = "atmega8")]
pub use avr_device::atmega8 as pac;

#[cfg(feature = "atmega1280")]
pub use avr_device::atmega1280 as pac;
#[cfg(feature = "atmega1284p")]
pub use avr_device::atmega1284p as pac;
#[cfg(feature = "atmega168")]
pub use avr_device::atmega168 as pac;
#[cfg(feature = "atmega2560")]
pub use avr_device::atmega2560 as pac;
#[cfg(feature = "atmega328p")]
pub use avr_device::atmega328p as pac;
#[cfg(feature = "atmega328pb")]
pub use avr_device::atmega328pb as pac;
#[cfg(feature = "atmega32u4")]
pub use avr_device::atmega32u4 as pac;
#[cfg(feature = "atmega48p")]
pub use avr_device::atmega48p as pac;

#[cfg(feature = "attiny167")]
pub use avr_device::attiny167 as pac;
#[cfg(feature = "attiny2313")]
pub use avr_device::attiny2313 as pac;
#[cfg(feature = "attiny84")]
pub use avr_device::attiny84 as pac;
#[cfg(feature = "attiny85")]
pub use avr_device::attiny85 as pac;
#[cfg(feature = "attiny88")]
pub use avr_device::attiny88 as pac;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
