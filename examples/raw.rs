#![no_std]
#![no_main]

use hal::pac;
use {milkv_duo_hal as hal, milkv_duo_riscv_rt as _, panic_halt as _};

#[milkv_duo_riscv_rt::entry]
fn main() -> ! {
    let pinmux = unsafe { &*pac::PINMUX::PTR };

    let pad = 0xac / 4; // pac in pinmux
    let pin = 0x34 / 4; // pin of ioblk
    pinmux.pad(pad).func_sel().write(|w| w.value().variant(0)); // PWR_GPIO[2]

    let pinctrl = unsafe { &*pac::IOBLK_RTC::PTR };
    pinctrl.pin(pin).iocfg().modify(|_, w| w.pu().set_bit());

    let gpio = unsafe { &*pac::PWR_GPIO::PTR };
    gpio.ddr().modify(|r, w| unsafe { w.bits(r.bits() | (1 << 2)) });

    loop {
        // gpio.dr()
        // .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 2)) });
        gpio.dr().modify(|r, w| unsafe { w.bits(r.bits() | (1 << 2)) });

        riscv::asm::delay(10_000_000);

        gpio.dr().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 2)) });

        riscv::asm::delay(10_000_000);
    }
}
