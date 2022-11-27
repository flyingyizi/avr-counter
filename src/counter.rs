//! macro to define

/// TC origin clock input is cpu-freq. prescale should be 1, 8, 64, 256, 1024. final TC freq
/// is  "CPU_FREQ / prescale".
///
/// judge whether TC can store input timeout . if yes return prescale and related ticks.
/// the ticks is the based on final prescaled TC freq. if can not store the timeout, return err
pub fn to_prescale_ticks(
    cpu_freq_hz: u32,
    timeout: fugit::MicrosDurationU32,
    max_ticks: u16,
) -> Result<(u16 /*prescale*/, u16 /*newticks*/), ()> {
    //only support Mhz
    debug_assert!(cpu_freq_hz >= 1_000_000);

    let cpu_freq = cpu_freq_hz / 1_000_000;

    let micros = timeout.ticks();

    let ticks_1 = micros * cpu_freq;

    let max_micros = max_ticks as u32 / cpu_freq;

    if micros <= max_micros {
        let newticks = ticks_1 as u16;
        return Ok((1, newticks));
    } else if micros <= max_micros * 8 {
        let newticks = (ticks_1 / 8) as u16;
        return Ok((8, newticks));
    } else if micros <= max_micros * 64 {
        let newticks = (ticks_1 / 64) as u16;
        return Ok((64, newticks));
    } else if micros <= max_micros * 256 {
        let newticks = (ticks_1 / 256) as u16;
        return Ok((256, newticks));
    } else if micros <= max_micros * 1024 {
        let newticks = (ticks_1 / 1024) as u16;
        return Ok((1024, newticks));
    }

    Err(())
}

#[macro_export]
macro_rules! impl_tc_traditional {
    (
        name: $Name:tt,
        peripheral: $tc:ty,
        start_ctc_mode: |$periph_ctcmode_var:ident, $prescale:ident, $ticks:ident| $start_ctc_mode:block,
        is_block: |$periph_wait_var:ident|->bool $wait:block,
        calc_overf: |$cpu_freq: ident, $timeout: ident|->Result<(u16/*prescale*/, u16/*ticks*/),()> $calc_overf:block,
    ) => {
        /// support embedded_hal::timer::CountDown. CPU_FREQ should be Mhz,e.g. 16_000_000.
        ///
        pub struct $Name<const CPU_FREQ: u32> {}
        impl<const CPU_FREQ: u32> $Name<CPU_FREQ> {
            #[inline]
            unsafe fn tc_set_ctcmode(&self, prescale: u16, ticks: u16) {
                let $periph_ctcmode_var = &(*<$tc>::ptr());
                let $prescale = prescale;
                let $ticks = ticks;

                $start_ctc_mode
            }
            #[inline]
            fn tc_calculate_overf(
                &self,
                timeout: $crate::fugit::MicrosDurationU32,
            ) -> Result<(u16 /*prescale*/, u16 /*ticks*/), ()> {
                let $cpu_freq = CPU_FREQ;
                let $timeout = timeout;
                $calc_overf
            }
            #[inline]
            unsafe fn tc_would_block(&self) -> bool {
                let $periph_wait_var = &(*<$tc>::ptr());
                $wait
            }

            /// Initialize timer
            pub fn new() -> Self {
                Self {}
            }
            fn _start(&mut self, timeout: $crate::fugit::MicrosDurationU32) -> Result<(), ()> {
                unsafe {
                    if let Ok((prescale, ticks)) = self.tc_calculate_overf(timeout) {
                        self.tc_set_ctcmode(prescale, ticks);
                    } else {
                        return Err(());
                    }
                }

                Ok(())
            }
            fn _wait(&mut self) -> $crate::nb::Result<(), $crate::void::Void> {
                unsafe {
                    if true == self.tc_would_block() {
                        return Err($crate::nb::Error::WouldBlock);
                    }
                }
                Ok(())
            }
        }

        impl<const CPU_FREQ: u32> $crate::embedded_hal::timer::CountDown for $Name<CPU_FREQ> {
            type Time = fugit::MicrosDurationU32;
            fn start<T>(&mut self, timeout: T)
            where
                T: Into<Self::Time>,
            {
                self._start(timeout.into()).unwrap()
            }
            fn wait(&mut self) -> nb::Result<(), void::Void> {
                self._wait()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_atmega_tc0 {
    (
        name: $Name:tt,
    ) => {
        $crate::impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC0,
            start_ctc_mode: |peripheral, prescale,ticks| {
                    //pause
                    peripheral.tccr0b.write(|w| w.cs0().no_clock() );

                    //reset
                    peripheral.tcnt0.write(|w|  w.bits(0) );
                    peripheral.ocr0a.write(|w|  w.bits(ticks as u8) );
                    // set CTC mode
                    // WGM02 WGM01 WGM00
                    //  0    1     0      CTC
                    peripheral.tccr0a.write(|w| w.wgm0().ctc());
                    peripheral.tccr0b.write(|w| {
                        //w.wgm02().clear_bit();
                        match prescale {
                            1 => w.cs0().direct(),
                            8 => w.cs0().prescale_8(),
                            64 => w.cs0().prescale_64(),
                            256 => w.cs0().prescale_256(),
                            1024 => w.cs0().prescale_1024(),
                            _ => {unreachable!()},
                        }

                    });

            },
            is_block: |peripheral| -> bool{
                if true == peripheral.tifr0.read().ocf0a().bit_is_set() {
                    peripheral.tifr0.modify(|_, w| w.ocf0a().set_bit());
                    return false;
                }
                true
            },
            calc_overf: |cpu_freq,timeout|->Result<(u16/*prescale*/, u16/*ticks*/),()> {
                $crate::to_prescale_ticks(cpu_freq,timeout, core::u8::MAX as u16)
            },
        }
    };
}

#[macro_export]
macro_rules! impl_atmega_tc1 {
    (
        name: $Name:tt,
    ) => {
        $crate::impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC1,
            start_ctc_mode: |peripheral, prescale,ticks| {
                    //pause
                    peripheral.tccr1b.write(|w| w.cs1().no_clock() );
                    //reset
                    peripheral.tcnt1.write(|w|  w.bits(0) );
                    peripheral.ocr1a.write(|w|  w.bits(ticks -1 as u16) );
                    // set CTC mode
                    // WGM13 WGM12 WGM11 WGM10
                    // 0     1     0     0     CTC
                    peripheral.tccr1a.write(|w| w.wgm1().bits(0b00));
                    peripheral.tccr1b.write(|w| {
                        w.wgm1().bits(0b01);
                        match prescale {
                            1 => w.cs1().direct(),
                            8 => w.cs1().prescale_8(),
                            64 => w.cs1().prescale_64(),
                            256 => w.cs1().prescale_256(),
                            1024 => w.cs1().prescale_1024(),
                            _ => {unreachable!()},
                        }

                    });

            },
            is_block: |peripheral| -> bool{
                if true == peripheral.tifr1.read().ocf1a().bit_is_set() {
                    // clear the flag bit manually since there is no ISR to execute
                    // clear it by writing '1' to it (as per the datasheet)
                    peripheral.tifr1.modify(|_, w| w.ocf1a().set_bit());
                    return false;
                }
                true
            },
            calc_overf: |cpu_freq,timeout|->Result<(u16/*prescale*/, u16/*ticks*/),()> {
                $crate::to_prescale_ticks(cpu_freq,timeout, core::u16::MAX)
            },

        }
    };
}

#[macro_export]
macro_rules! impl_atmega_tc2 {
    (
        name: $Name:tt,
    ) => {
        impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC2,
            start_ctc_mode: |peripheral, prescale,ticks| {
                    //pause
                    peripheral.tccr2b.write(|w| w.cs2().no_clock() );
                    //todo TC2 support 32, 128 prescale.

                    //reset
                    peripheral.tcnt2.write(|w|  w.bits(0) );
                    peripheral.ocr2a.write(|w|  w.bits( (ticks -1) as u8) );
                    // set CTC mode
                    // WGM2 WGM1 WGM0
                    // 0    1    0 CTC
                    peripheral.tccr2a.write(|w| w.wgm2().ctc());// .bits(0b10));
                    peripheral.tccr2b.write(|w| {
                        // w.wgm22().clear_bit();
                        match prescale {
                            1 => w.cs2().direct(),
                            8 => w.cs2().prescale_8(),
                            64 => w.cs2().prescale_64(),
                            256 => w.cs2().prescale_256(),
                            1024 => w.cs2().prescale_1024(),
                            _ => {unreachable!()},
                        }

                    });

            },
            is_block: |peripheral| -> bool{
                if true == peripheral.tifr2.read().ocf2a().bit_is_set() {
                    peripheral.tifr2.modify(|_, w| w.ocf2a().set_bit());
                    return false;
                }
                true
            },
            calc_overf: |cpu_freq,timeout|->Result<(u16/*prescale*/, u16/*ticks*/),()> {
                $crate::to_prescale_ticks(cpu_freq,timeout, core::u8::MAX as u16)
            },
        }
    };
}

#[cfg(any(
    feature = "atmega32u4",
    feature = "atmega48p",
    feature = "atmega168",
    feature = "atmega328p",
    feature = "atmega328pb",
    feature = "atmega2560",
    feature = "atmega1280",
    feature = "atmega1284p"
))]
impl_atmega_tc0! {
    name: Counter0,
}

#[cfg(any(
    feature = "atmega32u4",
    feature = "atmega48p",
    feature = "atmega168",
    feature = "atmega328p",
    feature = "atmega328pb",
    feature = "atmega2560",
    feature = "atmega1280",
    feature = "atmega1284p"
))]
impl_atmega_tc1! {
    name: Counter1,
}

// not include atmega32u4
#[cfg(any(
    feature = "atmega48p",
    feature = "atmega168",
    feature = "atmega328p",
    feature = "atmega328pb",
    feature = "atmega2560",
    feature = "atmega1280",
    feature = "atmega1284p"
))]
impl_atmega_tc2! {
    name: Counter2,
}

#[cfg(test)]
mod tests {
    use crate::to_prescale_ticks;
    // pub fn to_prescale_ticks(
    //     cpu_freq: u32,
    //     timeout: fugit::MicrosDurationU32,
    //     max_ticks: u16,
    // ) -> Result<(u16 /*prescale*/, u16 /*newticks*/), ()> {

    #[test]
    fn it_works() {
        let timeout = fugit::MicrosDurationU32::micros(5);
        let r = to_prescale_ticks(16_000_000, timeout, core::u8::MAX as u16);
        // println!("{:?}", r);
        assert_eq!(r, Ok((1, 80)));
        let timeout = fugit::MicrosDurationU32::micros(15000);
        let r = to_prescale_ticks(16_000_000, timeout, core::u8::MAX as u16);
        // println!("{:?}", r);
        assert_eq!(r, Ok((1024, 234)));
    }
}
