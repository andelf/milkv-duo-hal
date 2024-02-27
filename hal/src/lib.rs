#![no_std]

pub use milkv_duo_pac as pac;
pub use peripheral::*;

// macros come first
mod traits;

pub mod gpio;
pub mod uart;

// pub mod ddr;
pub mod signature;

mod peripheral;
pub mod peripherals;
pub mod sbi;

pub mod rom_api {
    extern "C" {
        fn p_rom_api_get_boot_src() -> u32;
    }

    #[derive(Debug)]
    #[repr(u32)]
    pub enum BootSrc {
        SpiNand = 0,
        SpiNor = 2,
        Emmc = 3,
        SdCard = 0xA0,
        Usb = 0xA3,
        Uart = 0xA5,
        Unknown(u32),
    }

    pub fn get_boot_src() -> BootSrc {
        unsafe {
            match p_rom_api_get_boot_src() & 0xFF {
                0 => BootSrc::SpiNand,
                2 => BootSrc::SpiNor,
                3 => BootSrc::Emmc,
                0xA0 => BootSrc::SdCard,
                0xA3 => BootSrc::Usb,
                0xA5 => BootSrc::Uart,
                v => BootSrc::Unknown(v),
            }
        }
    }
}

pub fn init() -> peripherals::Peripherals {
    peripherals::Peripherals::take()
}
