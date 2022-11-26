implement embedded_hal::timer::CountDown trait for avr TC

## how to review final codes

cargo-expand is a useful tool to review rust macro expand result.
```
$ cargo +nightly install cargo-expand
```
e.g. manual review  atmega328p Counter0/Counter1/Counter2 final codes


execute  `cargo expand`
```
avr-counter/$ cargo expand --features atmega328p counter
```
will get belows outputs:
```rust
...
mod counter {

    /// $Name support embedded_hal::timer::CountDown
    ///
    pub struct Counter0<const CPU_FREQ: u32> {}
    impl<const CPU_FREQ: u32> Counter0<CPU_FREQ> {
        #[inline]
        unsafe fn tc_set_ctcmode(&self, prescale: u16, ticks: u16) {
            let peripheral = &(*<crate::pac::TC0>::ptr());
            let prescale = prescale;
            let ticks = ticks;
            {
                peripheral
                    .tccr0b
                    .write(|w| {
                        w.cs0().variant(crate::pac::tc0::tccr0b::CS0_A::NO_CLOCK)
                    });
                let prescale = match prescale {
                    1 => crate::pac::tc0::tccr0b::CS0_A::DIRECT,
                    8 => crate::pac::tc0::tccr0b::CS0_A::PRESCALE_8,
                    64 => crate::pac::tc0::tccr0b::CS0_A::PRESCALE_64,
                    256 => crate::pac::tc0::tccr0b::CS0_A::PRESCALE_256,
                    1024 => crate::pac::tc0::tccr0b::CS0_A::PRESCALE_1024,
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                };
                peripheral.tccr0a.write(|w| w.wgm0().bits(0b10));
                peripheral.tccr0b.write(|w| { w.cs0().variant(prescale) });
                peripheral.tcnt0.write(|w| unsafe { w.bits(0) });
                peripheral.ocr0a.write(|w| unsafe { w.bits(ticks as u8) });
            }
        }
        #[inline]
        fn tc_calculate_overf(
            &self,
            timeout: crate::fugit::MicrosDurationU32,
        ) -> Result<(u16, u16), ()> {
            let cpu_freq = CPU_FREQ;
            let timeout = timeout;
            { crate::to_prescale_ticks(cpu_freq, timeout, core::u8::MAX as u16) }
        }
        #[inline]
        unsafe fn tc_would_block(&self) -> bool {
            let peripheral = &(*<crate::pac::TC0>::ptr());
            {
                if true == peripheral.tifr0.read().ocf0a().bit_is_set() {
                    peripheral.tifr0.modify(|_, w| w.ocf0a().set_bit());
                    return false;
                }
                true
            }
        }
        /// Initialize timer
        pub fn new() -> Self {
            Self {}
        }
        fn _start(
            &mut self,
            timeout: crate::fugit::MicrosDurationU32,
        ) -> Result<(), ()> {
            unsafe {
                if let Ok((prescale, ticks)) = self.tc_calculate_overf(timeout) {
                    self.tc_set_ctcmode(prescale, ticks);
                } else {
                    return Err(());
                }
            }
            Ok(())
        }
        fn _wait(&mut self) -> crate::nb::Result<(), crate::void::Void> {
            unsafe {
                if true == self.tc_would_block() {
                    return Err(crate::nb::Error::WouldBlock);
                }
            }
            Ok(())
        }
    }
....
```


## example

see examples dir
