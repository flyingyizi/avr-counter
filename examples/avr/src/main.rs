//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(default_alloc_error_handler)]

extern crate alloc;
use alloc::{format, string::String};
use arduino_hal::prelude::*;

use avr_counter::{prelude::*, Counter0, Counter1};

use avr_allocator::AvrHeap;
#[global_allocator]
static ALLOCATOR: AvrHeap = AvrHeap::empty();

use panic_halt as _;
#[arduino_hal::entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 256;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    ufmt::uwriteln!(&mut serial, "Hello from Arduino! simulate Counter1:\r").void_unwrap();

    let mut x_step = pins.d2.into_output().downgrade();

    const CPUFREQ: u32 = 16_000_000; //16MHz
    let mut counter = Counter0::<{ CPUFREQ }>::new();

    let pluse_length = fugit::MicrosDurationU32::micros(10);
    let delay = pluse_length * 10;

    let o = format!("pluse {:?} delay {:?}", pluse_length, delay);
    ufmt::uwriteln!(&mut serial, "{}:\r", o.as_str()).void_unwrap();

    // ufmt::uwriteln!(&mut serial, "1:\r").void_unwrap();
    //     let _ = counter.start(delay);
    //     let _ = x_step .set_high();
    //     let _ = nb::block!(counter.wait());
    // ufmt::uwriteln!(&mut serial, "2:\r").void_unwrap();
    
    loop {
        let _ = x_step.set_high();
        let _ = counter.start(delay);
        let _ = nb::block!(counter.wait());

        let _ = x_step.set_low();
        let _ = counter.start(pluse_length);
        let _ = nb::block!(counter.wait());
    }
}
