//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;

use avr_counter::{Counter1,Counter0,prelude::*};

use avr_allocator::AvrHeap;

#[global_allocator]
static ALLOCATOR: AvrHeap = AvrHeap::empty();

// use fugit::{TimerDurationU32, TimerDurationU64};

use arduino_hal::clock::Clock;
use panic_halt as _;
#[arduino_hal::entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    ufmt::uwriteln!(&mut serial, "Hello from Arduino! simulate Counter1:\r").void_unwrap();

    let mut x_step = pins.d2.into_output().downgrade();

    // const CPUFREQ: u32 = arduino_hal::DefaultClock::FREQ;
    const CPUFREQ: u32 = 16_000_000; //16MHz
    let mut counter = Counter1::<{ CPUFREQ }>::new();
    ufmt::uwriteln!(&mut serial, "generated Counter1:\r").void_unwrap();

    // 100u     300u 900u
    let pluse_length = fugit::MicrosDurationU32::micros(5);
    let delay = fugit::MicrosDurationU32::micros(100);
    // let delay =  fugit::MicrosDurationU32::micros(15);

    let peripheral = unsafe{&(*arduino_hal::pac::TC1::ptr())};
    use arduino_hal::pac::tc1::tccr1b::CS1_A;
    loop {
        peripheral.tccr1b.write(|w| w.cs1().variant( CS1_A::NO_CLOCK));
        let _ = counter.start(delay);
        let _ = x_step.set_high();
        let _ = nb::block!(counter.wait());

        peripheral.tccr1b.write(|w| w.cs1().variant( CS1_A::NO_CLOCK));

        let _ = counter.start(pluse_length);
        let _ = x_step.set_low();
        let _ = nb::block!(counter.wait());
    }
}
