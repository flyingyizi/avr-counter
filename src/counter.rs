//! macro to define


use core::marker;
use embedded_hal::timer::CountDown;

#[derive(ufmt::derive::uDebug, Debug,, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Timer is disabled
    Disabled,
    WrongAutoReload,
}

bitflags::bitflags! {
    pub struct Event: u32 {
        const Update  = 1 << 0; //Overflow
        const InputCapture = 1 << 1;
        const MatchOCR = 1 << 2;
        const C3 = 1 << 3;
        const C4 = 1 << 4;
    }
}

/// Timer wrapper
pub struct Timer<TIM> {
    pub(crate) tim: TIM,
    pub(crate) clk: u32, //hz
}

pub trait CounterOps<H,T> {
    type Width: Into<u32> + From<u16> + From<u8>;
    fn max_auto_reload() -> u32;
    unsafe fn set_auto_reload_unchecked(&mut self, arr: u32);
    fn set_auto_reload(&mut self, arr: u32) -> Result<(), super::Error>;
    fn read_auto_reload() -> u32;
    fn enable_preload(&mut self, b: bool);
    fn enable_counter(&mut self);
    fn disable_counter(&mut self);
    fn is_counter_enabled(&self) -> bool;
    fn reset_counter(&mut self);
    fn set_prescaler(&mut self, psc: u16);
    fn read_prescaler(&self) -> u16;
    fn trigger_update(&mut self);
    fn clear_interrupt_flag(&mut self, event: Event);
    fn listen_interrupt(&mut self, event: Event, b: bool);
    fn get_interrupt_flag(&self) -> Event;
    fn read_count(&self) -> Self::Width;
    fn start_one_pulse(&mut self);
    fn start_no_update(&mut self);
    fn cr1_reset(&mut self);
}

// // All F4xx parts have these timers.
// hal!(
//     pac::TIM9: [Timer9, u16, c: (CH2),],
//     pac::TIM11: [Timer11, u16, c: (CH1),],
// );
macro_rules! hal {
    ($($TIM:ty: [
        $Timer:ident,
        $bits:ty,
        regs: [($tccra:tt,$tccrb:tt, $ocra:tt, $tcnt:tt, $tifr:tt),($cs:tt, $ocfa:tt)] ,

        // $(c: ($cnum:ident $(, $aoe:ident)?),)?
        // $(m: $timbase:ident,)?
    ],)+) => {
        $(
            pub type $Timer = Timer<$TIM>;

            impl General for $TIM {
                type Width = $bits;

                #[inline(always)]
                fn max_auto_reload() -> u32 {
                    <$bits>::MAX as u32
                }
                #[inline(always)]
                unsafe fn set_auto_reload_unchecked(&mut self, arr: u32) {
                    self.$tcnt.write(|w| w.bits(arr as Self::Width));
                }

                #[inline(always)]
                fn set_auto_reload(&mut self, arr: u32) -> Result<(), Error> {
                    // Note: Make it impossible to set the ARR value to 0, since this
                    // would cause an infinite loop.
                    if arr > 0 && arr <= Self::max_auto_reload() {
                        Ok(unsafe { self.set_auto_reload_unchecked(arr) })
                    } else {
                        Err(Error::WrongAutoReload)
                    }
                }
                
                #[inline(always)]
                fn read_auto_reload() -> u32 {
                    let tim = unsafe { &*<$TIM>::ptr() };
                    tim.arr.read().bits()
                }
                #[inline(always)]
                fn enable_preload(&mut self, b: bool) {
                    // self.cr1.modify(|_, w| w.arpe().bit(b));
                }
                #[inline(always)]
                fn enable_counter(&mut self) {
                    self.cr1.modify(|_, w| w.cen().set_bit());
                }
                #[inline(always)]
                fn disable_counter(&mut self) {
                    self.cr1.modify(|_, w| w.cen().clear_bit());
                }
                #[inline(always)]
                fn is_counter_enabled(&self) -> bool {
                    self.cr1.read().cen().is_enabled()
                }
                #[inline(always)]
                fn reset_counter(&mut self) {
                    self.cnt.reset();
                }
                #[inline(always)]
                fn set_prescaler(&mut self, psc: u16) {
                    self.psc.write(|w| w.psc().bits(psc) );
                }
                #[inline(always)]
                fn read_prescaler(&self) -> u16 {
                    self.psc.read().psc().bits()
                }
                #[inline(always)]
                fn trigger_update(&mut self) {
                    self.cr1.modify(|_, w| w.urs().set_bit());
                    self.egr.write(|w| w.ug().set_bit());
                    self.cr1.modify(|_, w| w.urs().clear_bit());
                }
                #[inline(always)]
                fn clear_interrupt_flag(&mut self, event: Event) {
                    self.sr.write(|w| unsafe { w.bits(0xffff & !event.bits()) });
                }
                #[inline(always)]
                fn listen_interrupt(&mut self, event: Event, b: bool) {
                    if b {
                        self.dier.modify(|r, w| unsafe { w.bits(r.bits() | event.bits()) });
                    } else {
                        self.dier.modify(|r, w| unsafe { w.bits(r.bits() & !event.bits()) });
                    }
                }
                #[inline(always)]
                fn get_interrupt_flag(&self) -> Event {
                    Event::from_bits_truncate(self.sr.read().bits())
                }
                #[inline(always)]
                fn read_count(&self) -> Self::Width {
                    self.cnt.read().bits() as Self::Width
                }
                #[inline(always)]
                fn start_one_pulse(&mut self) {
                    self.cr1.write(|w| unsafe { w.bits(1 << 3) }.cen().set_bit());
                }
                #[inline(always)]
                fn start_no_update(&mut self) {
                    self.cr1.write(|w| w.cen().set_bit().udis().set_bit());
                }
                #[inline(always)]
                fn cr1_reset(&mut self) {
                    self.cr1.reset();
                }
            }
            $(with_pwm!($TIM: $cnum $(, $aoe)?);)?

            $(impl MasterTimer for $TIM {
                type Mms = pac::$timbase::cr2::MMS_A;
                fn master_mode(&mut self, mode: Self::Mms) {
                    self.cr2.modify(|_,w| w.mms().variant(mode));
                }
            })?
        )+
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////////////////
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

                //set to ctc mode
                {
                    let $periph_ctcmode_var = &(*<$tc>::ptr());
                    $start_ctc_mode
                }

                peripheral.$tccrb.write(|w| match prescale {
                    1 => w.$cs().direct(),
                    8 => w.$cs().prescale_8(),
                    64 => w.$cs().prescale_64(),
                    256 => w.$cs().prescale_256(),
                    1024 => w.$cs().prescale_1024(),
                    _ => {
                        unreachable!()
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
                debug_assert!(CPU_FREQHZ >= 1_000_000);

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

            /// Initialize timer
            pub fn new() -> Self {
                Self {}
            }
        }

        impl<const CPU_FREQHZ: u32> $crate::embedded_hal::timer::CountDown for $Name<CPU_FREQHZ> {
            type Time = fugit::MicrosDurationU32;
            fn start<T>(&mut self, timeout: T)
            where
                T: Into<Self::Time>,
            {
                if let Some((prescale, ticks)) = self.tc_calculate_overf(timeout.into()) {
                    unsafe { self.tc_init(prescale, ticks) };
                }
            }
            fn wait(&mut self) -> $crate::nb::Result<(), $crate::void::Void> {
                unsafe {
                    let peripheral = &(*<$tc>::ptr());
                    if true == peripheral.$tifr.read().$ocfa().bit_is_set() {
                        peripheral.$tifr.modify(|_, w| w.$ocfa().set_bit());
                        return Ok(());
                    }
                }
                Err($crate::nb::Error::WouldBlock)
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

#[macro_export]
macro_rules! impl_atmega_tc3 {
    (
        name: $Name:tt,
    ) => {
        impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC3,
            bits:u16,
            start_ctc_mode: |peripheral| {
                    // set CTC mode
                    // WGM3[3] WGM3[2] WGM3[1] WGM3[0]
                    // 0       1       0       0        CTC
                    peripheral.tccr3a.write(|w| w.wgm3().bits(0b00));
                    peripheral.tccr3b.write(|w| w.wgm3().bits(0b01));
            },
            regs: [(tccr3a,tccr3b, ocr3a, tcnt3,tifr3),(cs3, ocf3a)] ,
        }
    };
}

#[macro_export]
macro_rules! impl_atmega_tc4 {
    (
        name: $Name:tt,
    ) => {
        impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC4,
            bits:u16,
            start_ctc_mode: |peripheral| {
                    // set CTC mode
                    // WGM4[3] WGM4[2] WGM4[1] WGM4[0]
                    // 0       1       0       0        CTC
                    peripheral.tccr4a.write(|w| w.wgm4().bits(0b00));
                    peripheral.tccr4b.write(|w| w.wgm4().bits(0b01));
            },
            regs: [(tccr4a,tccr4b, ocr4a, tcnt4,tifr4),(cs4, ocf4a)] ,
        }
    };
}
#[macro_export]
macro_rules! impl_atmega_tc5 {
    (
        name: $Name:tt,
    ) => {
        impl_tc_traditional! {
            name: $Name,
            peripheral: $crate::pac::TC5,
            bits:u16,
            start_ctc_mode: |peripheral| {
                    // set CTC mode
                    // WGM5[3] WGM5[2] WGM5[1] WGM5[0]
                    // 0       1       0       0        CTC
                    peripheral.tccr5a.write(|w| w.wgm5().bits(0b00));
                    peripheral.tccr5b.write(|w| w.wgm5().bits(0b01));
            },
            regs: [(tccr5a,tccr5b, ocr5a, tcnt5,tifr5),(cs5, ocf5a)] ,
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

#[cfg(any(
    feature = "atmega32u4",
    feature = "atmega328pb",
    feature = "atmega2560",
    feature = "atmega1280",
    feature = "atmega1284p"
))]
impl_atmega_tc3! {
    name: Counter3,
}

#[cfg(any(
    feature = "atmega32u4",
    feature = "atmega328pb",
    feature = "atmega2560",
    feature = "atmega1280"
))]
impl_atmega_tc4! {
    name: Counter4,
}
#[cfg(any(feature = "atmega2560", feature = "atmega1280"))]
impl_atmega_tc5! {
    name: Counter5,
}

//[ATMEGA328PB datasheet](https://datasheet.lcsc.com/lcsc/2210181030_Microchip-Tech-ATMEGA328PB-AU_C132230.pdf)
//[ATMEGA8A datasheet](https://datasheet.lcsc.com/lcsc/2210141830_Microchip-Tech-ATMEGA8A-AN_C5127755.pdf)
#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
