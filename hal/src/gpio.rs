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

        pin.set_as_gpio();

        Self { pin: pin.map_into() }
    }

    #[inline]
    pub fn set_as_input(&mut self) {
        self.pin
            .block()
            .ddr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << self.pin.pin())) });
    }

    #[inline]
    pub fn set_as_output(&mut self) {
        self.pin
            .block()
            .ddr()
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << self.pin.pin())) });
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.block().ext_port().read().bits() & (1 << self.pin.pin()) == 0
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
        /// The u32 to represent the pin
        /// [31:24] - FMUX pad offset
        /// [23:16] - IOBLK group, 1, 7, 10, 12, 13(PWR/RTC)
        /// [15:8] - IOBLK pin num
        /// [7:5] - IO port, 0, 1, 2, 3, 4(PWR)
        /// [4:0] - IO number, 0 to 32
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

    /// Set pinmux
    #[inline]
    fn set_alt_function(&self, func: u8) {
        self.fmux().func_sel().write(|w| w.value().variant(func));
    }

    #[inline]
    fn set_as_gpio(&self) {
        // PWR_GPIO[0], PWR_GPIO[1], PWR_GPIO[2] are special case for alt functions settings
        // On MilkV Duo 250m board, PWR_GPIO[2] is used for LED, the other two are NC
        if self._ioport() == 4 && self._ionum() <= 2 {
            self.set_alt_function(0);
        } else {
            self.set_alt_function(3); // XGPIO[x] alt functions
        }
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

// PIN, FMUX_PAD, IOBLK_GROUP, IOBLK_PIN, IO_PORT, IO_PIN
impl_pin!(PIN_0, 0x70 / 4, 7, 0x3c / 4, 0, 28); // 3 : XGPIOA[28], IIC0_SCL
impl_pin!(PIN_1, 0x74 / 4, 7, 0x40 / 4, 0, 29); // 3 : XGPIOA[29], IIC0_SDA
impl_pin!(PIN_2, 0x64 / 4, 7, 0x30 / 4, 0, 19); // 3 : XGPIOA[19], JTAG_CPU_TMS
impl_pin!(PIN_3, 0x68 / 4, 7, 0x34 / 4, 0, 18); // 3 : XGPIOA[18], JTAG_CPU_TCK

impl_pin!(PIN_4, 0xD4 / 4, 13, 0x5C / 4, 4, 19); // 3 : PWR_GPIO[19], SD1_D2
impl_pin!(PIN_5, 0xD8 / 4, 13, 0x60 / 4, 4, 20); // 3 : PWR_GPIO[20], SD1_D1
impl_pin!(PIN_6, 0xE4 / 4, 13, 0x6C / 4, 4, 23); // 3 : PWR_GPIO[23], SD1_CLK
impl_pin!(PIN_7, 0xE9 / 4, 13, 0x68 / 4, 4, 22); // 3 : PWR_GPIO[22], SD1_CMD
impl_pin!(PIN_8, 0xDC / 4, 13, 0x64 / 4, 4, 21); // 3 : PWR_GPIO[21], SD1_D0
impl_pin!(PIN_9, 0xD0 / 4, 13, 0x58 / 4, 4, 18); // 3 : PWR_GPIO[18], SD1_D3

impl_pin!(PIN_10, 0x1AC / 4, 12, 0x78 / 4, 2, 14); // 3 : XGPIOC[14], I2C_SDA, PAD_MIPI_TXM1
impl_pin!(PIN_11, 0x1B0 / 4, 12, 0x7C / 4, 2, 15); // 3 : XGPIOC[15], I2C_SCL, PAD_MIPI_TXP1

impl_pin!(PIN_12, 0x40 / 4, 7, 0x0C / 4, 0, 16); // 3 : XGPIOA[16], UART0_TX
impl_pin!(PIN_13, 0x44 / 4, 7, 0x10 / 4, 0, 17); // 3 : XGPIOA[17], UART0_RX

impl_pin!(PIN_14, 0x38 / 4, 7, 0x04 / 4, 0, 14); // 3 : XGPIOA[14], SD0_PWR_EN
impl_pin!(PIN_15, 0x3C / 4, 7, 0x08 / 4, 0, 15); // 3 : XGPIOA[15], SPK_EN

impl_pin!(PIN_16, 0x5C / 4, 7, 0x28 / 4, 0, 23); // 3 : XGPIOA[23], EMMC_CMD
impl_pin!(PIN_17, 0x60 / 4, 7, 0x2C / 4, 0, 24); // 3 : XGPIOA[24], EMMC_DAT1

impl_pin!(PIN_18, 0x50 / 4, 7, 0x1C / 4, 0, 22); // 3 : XGPIOA[22], EMMC_CLK
impl_pin!(PIN_19, 0x54 / 4, 7, 0x20 / 4, 0, 25); // 3 : XGPIOA[25], EMMC_DAT0
impl_pin!(PIN_20, 0x58 / 4, 7, 0x24 / 4, 0, 27); // 3 : XGPIOA[27], EMMC_DAT3
impl_pin!(PIN_21, 0x4C / 4, 7, 0x18 / 4, 0, 26); // 3 : XGPIOA[26], EMMC_DAT2
impl_pin!(PIN_22, 0x88 / 4, 13, 0x0C / 4, 4, 4); // 3 : PWR_GPIO[4], PWR_SEQ2

impl_pin!(PIN_25, 0xAC / 4, 13, 0x34 / 4, 4, 2); // 0: PWR_GPIO[2], LED

// Audio pins
impl_pin!(PIN_MIC_IN, 0x1BC / 4, 0, 0x00 / 4, 2, 23); // no ioblk
impl_pin!(PIN_AUDIO_OUT, 0x1C8 / 4, 0, 0x00 / 4, 2, 24); // no ioblk

// SD card, SDIO0
impl_pin!(PIN_SD0_CLK, 0x1C / 4, 10, 0x00 / 4, 0, 7); // 3 : XGPIOA[7], SD0_CLK
impl_pin!(PIN_SD0_CMD, 0x20 / 4, 10, 0x04 / 4, 0, 8); // 3 : XGPIOA[8], SD0_CMD
impl_pin!(PIN_SD0_D0, 0x24 / 4, 10, 0x08 / 4, 0, 9); // 3 : XGPIOA[9], SD0_D0
impl_pin!(PIN_SD0_D1, 0x28 / 4, 10, 0x0C / 4, 0, 10); // 3 : XGPIOA[10], SD0_D1
impl_pin!(PIN_SD0_D2, 0x2C / 4, 10, 0x10 / 4, 0, 11); // 3 : XGPIOA[11], SD0_D2
impl_pin!(PIN_SD0_D3, 0x30 / 4, 10, 0x14 / 4, 0, 12); // 3 : XGPIOA[12], SD0_D3
impl_pin!(PIN_SD0_CD, 0x34 / 4, 7, 0x00 / 4, 0, 13); // 3 : XGPIOA[13], SD0_CD

// ARM RV SWITCH pin, pulled up externally
impl_pin!(PIN_ARM_RV_SWITCH, 0x1CC / 4, 12, 0x8C / 4, 1, 23); // 3 : XGPIOB[23], ARM_RV_SWITCH, GPIO_RTX___EPHY_RTX

// 1V8 domain
impl_pin!(PIN_26, 0xF8 / 4, 1, 0x10 / 4, 1, 3); // 3 : XGPIOB[3], ADC1
impl_pin!(PIN_27, 0x108 / 4, 1, 0x20 / 4, 1, 6); // 3 : XGPIOB[6], ADC2, USB_VBUS_DET
