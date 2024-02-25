#![feature(abi_riscv_interrupt)]
#![no_std]
#![no_main]

use hal::gpio::Flex;
use hal::println;
use milkv_duo_hal as hal;

const BANNER: &str = r#"
 __  __ _ _ _      __     __  ____
|  \/  (_) | | __  \ \   / / |  _ \ _   _  ___
| |\/| | | | |/ /___\ \ / /  | | | | | | |/ _ \
| |  | | | |   <_____\ V /   | |_| | |_| | (_) |
|_|  |_|_|_|_|\_\     \_/    |____/ \__,_|\___/ "#;

#[no_mangle]
pub unsafe extern "riscv-interrupt-s" fn default_start_trap() {
    let stval = riscv::register::stvec::read().bits();
    let scause = riscv::register::scause::read();
    let sscratch = riscv::register::sscratch::read();

    crate::println!("stval: {:#x}", stval);
    crate::println!("scause: {:?}", scause.bits());
    crate::println!("sscratch: {:#x}", sscratch);

    loop {}
}

#[milkv_duo_riscv_rt::entry]
fn main() -> ! {
    hal::uart::Uart0::init();

    println!("{}", BANNER);
    println!("Boot Src: {:?}", hal::rom_api::get_boot_src());
    let chip_info = hal::signature::read_chip_info();
    println!("Chip info: {:x?}", chip_info);

    let p = hal::init();

    let mut led = Flex::new(p.PIN_25);
    led.set_as_output();
    led.set_high();

    println!("Hello world!!!!");

    loop {
        println!("toggle!!!!");
        led.toggle();

        riscv::asm::delay(28 * 10_000_000);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {:?}", info);
    loop {}
}
