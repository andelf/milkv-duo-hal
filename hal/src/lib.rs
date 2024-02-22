#![no_std]

pub use milkv_duo_pac as pac;
pub use peripheral::*;

mod peripheral;
pub mod peripherals;
pub mod uart;

pub mod ddr;

pub mod rom_api {
    extern "C" {
        pub fn p_rom_api_get_boot_src() -> u32;
    }
}
