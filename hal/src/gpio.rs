//! GPIO

use crate::gpio::sealed::Pin as _Pin;
use crate::{impl_peripheral, into_ref, pac, peripherals, Peripheral, PeripheralRef};

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Level {
    /// Logical low.
    Low,
    /// Logical high.
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// Represents a pull setting for an input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor. 100kOhm
    Down,
}

/// Slew rate of an output
#[derive(Debug, Eq, PartialEq)]
pub enum SlewRate {
    /// Fast slew rate.
    Fast = 0,
    /// Slow slew rate.
    Slow = 1,
}

/// GPIO flexible pin.
pub struct Flex<'d> {
    pin: PeripheralRef<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    #[inline]
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd) -> Self {
        into_ref!(pin);

        // TODO IOC selection
        pin.set_alt_function(0); // FIXME, some gpio are not 0

        Self { pin: pin.map_into() }
    }

    #[inline]
    pub fn set_as_output(&mut self) {
        self.pin
            .block()
            .ddr()
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << self.pin.pin())) });
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.pin
            .block()
            .dr()
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << self.pin.pin())) });
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.pin
            .block()
            .dr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << self.pin.pin())) });
    }

    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.block().dr().read().bits() & (1 << self.pin.pin()) == 0
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.pin
            .block()
            .dr()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << self.pin.pin())) });
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin: Sized {
        fn pad_pin_io_num(&self) -> u32;

        // FMUX pad
        #[inline]
        fn _pad(&self) -> usize {
            (self.pad_pin_io_num() >> 24) as usize
        }

        #[inline]
        fn _group(&self) -> usize {
            ((self.pad_pin_io_num() >> 16) & 0xff) as usize
        }

        #[inline]
        fn _pin(&self) -> usize {
            ((self.pad_pin_io_num() >> 8) & 0xff) as usize
        }

        #[inline]
        fn _ioport(&self) -> usize {
            ((self.pad_pin_io_num() & 0xff) as usize) >> 5
        }

        #[inline]
        fn _ionum(&self) -> u8 {
            (self.pad_pin_io_num() & 0x1f) as u8
        }

        #[inline]
        fn fmux(&self) -> &'static pac::pinmux::pad::PAD {
            let pinmux = unsafe { &*pac::PINMUX::PTR };
            pinmux.pad(self._pad())
        }

        #[inline]
        fn ctrl(&self) -> &'static pac::ioblk_g1::PIN {
            match self._group() {
                1 => unsafe { &*pac::IOBLK_G1::PTR }.pin(self._pin()),
                7 => unsafe { &*pac::IOBLK_G7::PTR }.pin(self._pin()),
                10 => unsafe { &*pac::IOBLK_G10::PTR }.pin(self._pin()),
                12 => unsafe { &*pac::IOBLK_G12::PTR }.pin(self._pin()),
                13 => unsafe { &*pac::IOBLK_RTC::PTR }.pin(self._pin()),
                _ => unreachable!(),
            }
        }

        #[inline]
        fn block(&self) -> &'static pac::gpio0::RegisterBlock {
            match self._ioport() {
                0 => unsafe { &*pac::GPIO0::PTR },
                1 => unsafe { &*pac::GPIO1::PTR },
                2 => unsafe { &*pac::GPIO2::PTR },
                3 => unsafe { &*pac::GPIO3::PTR },
                4 => unsafe { &*pac::PWR_GPIO::PTR },
                _ => unreachable!(),
            }
        }
    }
}

pub trait Pin: Peripheral<P = Self> + Into<AnyPin> + sealed::Pin + Sized + 'static {
    /// Degrade to a generic pin struct
    fn degrade(self) -> AnyPin {
        AnyPin {
            pad_pin_io_num: self.pad_pin_io_num(),
        }
    }

    /// Returns the pin number within a bank
    #[inline]
    fn pin(&self) -> u8 {
        self._ionum() as u8
    }

    /// Returns the bank of this pin
    #[inline]
    fn bank(&self) -> u8 {
        self._ioport() as u8
    }

    #[inline]
    fn set_alt_function(&self, func: u8) {
        self.fmux().func_sel().write(|w| w.value().variant(func));
    }

    #[inline]
    fn set_pull(&self, pull: Pull) {
        self.ctrl().iocfg().modify(|_, w| match pull {
            Pull::None => w.pu().clear_bit().pd().clear_bit(),
            Pull::Up => w.pu().set_bit().pd().clear_bit(),
            Pull::Down => w.pu().clear_bit().pd().set_bit(),
        });
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pad_pin_io_num: u32,
}

impl AnyPin {
    /// Unsafely create a new type-erased pin.
    ///
    /// # Safety
    ///
    /// You must ensure that you’re only using one instance of this type at a time.
    pub unsafe fn steal(pad_pin_io_num: u32) -> Self {
        Self { pad_pin_io_num }
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    fn pad_pin_io_num(&self) -> u32 {
        self.pad_pin_io_num
    }
}

macro_rules! impl_pin {
    ($name:ident, $fmux_pad_index:expr, $ioblk_group:expr, $ioblk_pin_index:expr, $io_port:expr, $io_pin:expr) => {
        impl Pin for peripherals::$name {}
        impl sealed::Pin for peripherals::$name {
            #[inline]
            fn pad_pin_io_num(&self) -> u32 {
                ($fmux_pad_index << 24) | ($ioblk_group << 16) | ($ioblk_pin_index << 8) | ($io_port << 5) | $io_pin
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                crate::gpio::Pin::degrade(val)
            }
        }
    };
}

impl_pin!(PIN_25, 0xAC / 4, 13, 0x34 / 4, 4, 2); // PWR_GPIO[2]
