//! macro to define

#[macro_export]
macro_rules! impl_tc_traditional {
    (
        name: $Name:tt,
        peripheral: $tc:ty,
        bits: $bits:ty,
        start_ctc_mode: |$periph_ctcmode_var:ident| $start_ctc_mode:block,
        regs: [($tccra:tt,$tccrb:tt, $ocra:tt, $tcnt:tt, $tifr:tt),($cs:tt, $ocfa:tt)] ,
    ) => {
        /// support embedded_hal::timer::CountDown. CPU_FREQHZ should be Mhz,e.g. 16_000_000.
        ///
        pub struct $Name<const CPU_FREQHZ: u32> {}
        impl<const CPU_FREQHZ: u32> $Name<CPU_FREQHZ> {
            #[inline]
            unsafe fn tc_init(&self, prescale: u16, ticks: $bits) {
                let peripheral = &(*<$tc>::ptr());
                //pause
                peripheral.$tccrb.write(|w| w.$cs().no_clock());

                //reset
                peripheral.$tcnt.write(|w| w.bits(0));
                peripheral.$ocra.write(|w| w.bits(ticks - 1));

                self.tc_set_ctcmode();

                peripheral.$tccrb.write(|w| {
                    match prescale {
                        1 => w.$cs().direct(),
                        8 => w.$cs().prescale_8(),
                        64 => w.$cs().prescale_64(),
                        256 => w.$cs().prescale_256(),
                        1024 => w.$cs().prescale_1024(),
                        _ => {
                            unreachable!()
                        }
                    }
                });
            }

            /// TC origin clock input is cpu-freq. prescale should be 1, 8, 64, 256, 1024. final TC freq
            /// is  "CPU_FREQHZ / prescale".
            ///
            /// judge whether TC can store input timeout . if yes return prescale and related ticks.
            /// the ticks is the based on final prescaled TC freq.
            fn tc_calculate_overf(
                &self,
                timeout: fugit::MicrosDurationU32,
            ) -> Option<(u16 /*prescale*/, $bits /*newticks*/)> {
                type Width = $bits;
                //only support Mhz
                // debug_assert!(cpu_freq_hz >= 1_000_000);

                let cpu_freq = CPU_FREQHZ / 1_000_000;

                let micros = timeout.ticks();

                let ticks_1 = micros * cpu_freq;

                let max_micros = Width::MAX as u32 / cpu_freq;

                if micros <= max_micros {
                    let newticks = ticks_1 as Width;
                    return Some((1, newticks));
                } else if micros <= max_micros * 8 {
                    let newticks = (ticks_1 / 8) as Width;
                    return Some((8, newticks));
                } else if micros <= max_micros * 64 {
                    let newticks = (ticks_1 / 64) as Width;
                    return Some((64, newticks));
                } else if micros <= max_micros * 256 {
                    let newticks = (ticks_1 / 256) as Width;
                    return Some((256, newticks));
                } else if micros <= max_micros * 1024 {
                    let newticks = (ticks_1 / 1024) as Width;
                    return Some((1024, newticks));
                }

                None
            }

            #[inline]
            unsafe fn tc_set_ctcmode(&self) {
                let $periph_ctcmode_var = &(*<$tc>::ptr());
                $start_ctc_mode
            }
            #[inline]
            unsafe fn tc_would_block(&self) -> bool {
                let peripheral = &(*<$tc>::ptr());

                if true == peripheral.$tifr.read().$ocfa().bit_is_set() {
                    peripheral.$tifr.modify(|_, w| w.$ocfa().set_bit());
                    return false;
                }
                true
            }

            /// Initialize timer
            pub fn new() -> Self {
                Self {}
            }
            fn _start(&mut self, timeout: $crate::fugit::MicrosDurationU32) -> Result<(), ()> {
                unsafe {
                    if let Some((prescale, ticks)) = self.tc_calculate_overf(timeout) {
                        self.tc_init(prescale, ticks);
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

        impl<const CPU_FREQHZ: u32> $crate::embedded_hal::timer::CountDown for $Name<CPU_FREQHZ> {
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
            bits:u8,
            start_ctc_mode: |peripheral| {
                    // set CTC mode
                    // WGM02 WGM01 WGM00
                    //  0    1     0      CTC
                    peripheral.tccr0a.write(|w| w.wgm0().ctc());
                    // peripheral.tccr0b.write(|w| w.wgm02().clear_bit());
            },
            regs: [(tccr0a,tccr0b, ocr0a, tcnt0,tifr0),(cs0, ocf0a)] ,
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
            bits:u16,
            start_ctc_mode: |peripheral| {

                // set CTC mode
                    // WGM13 WGM12 WGM11 WGM10
                    // 0     1     0     0     CTC
                    peripheral.tccr1a.write(|w| w.wgm1().bits(0b00));
                    peripheral.tccr1b.write(|w| w.wgm1().bits(0b01));

            },
            regs: [(tccr1a,tccr1b, ocr1a, tcnt1,tifr1),(cs1, ocf1a)] ,

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
            bits:u8,
            start_ctc_mode: |peripheral| {
                    // set CTC mode
                    // WGM2 WGM1 WGM0
                    // 0    1    0 CTC
                    peripheral.tccr2a.write(|w| w.wgm2().ctc());// .bits(0b10));
                    // peripheral.tccr2b.write(|w| w.wgm22().clear_bit());

            },
            regs: [(tccr2a,tccr2b, ocr2a, tcnt2,tifr2),(cs2, ocf2a)] ,
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
