use crate::pac;

/// Uart0 for debug
pub struct Uart0;

impl Uart0 {
    pub fn init() {
        let pinmux = unsafe { &*pac::PINMUX::PTR };
        // GP12 UART0_TX,
        // GP13 UART0_RX,
        let gp12_pad = 0x40 / 4;
        let gp13_pad = 0x44 / 4;
        let gp12_pin = 0x0c / 4;
        let gp13_pin = 0x10 / 4;
        let ioblk = unsafe { &*pac::IOBLK_G7::PTR };

        pinmux.pad(gp12_pad).func_sel().write(|w| w.value().variant(0));
        pinmux.pad(gp13_pad).func_sel().write(|w| w.value().variant(0));

        // set pull down
        ioblk.pin(gp12_pin).iocfg().modify(|_, w| w.pd().set_bit());
        ioblk.pin(gp13_pin).iocfg().modify(|_, w| w.pd().set_bit());

        // uart_clock / (16 * baudrate);
        let lpdl: u32 = 14;

        let uart = unsafe { &*pac::UART0::PTR };
        uart.lcr().modify(|_, w| w.dlab().set_bit());
        uart.lpdll().write(|w| w.lpdll().variant((lpdl & 0xff) as u8));
        uart.lpdlh().write(|w| w.lpdlh().variant(((lpdl >> 8) & 0xff) as u8));
        uart.lcr().modify(|_, w| w.dlab().clear_bit());

        // 8N1
        uart.lcr()
            .modify(|_, w| unsafe { w.stb().clear_bit().pen().clear_bit().wls().bits(0b11) });
    }

    pub fn write_byte(&mut self, data: u8) {
        let uart = unsafe { &*pac::UART0::PTR };
        while uart.lsr().read().thre().bit_is_clear() {}
        uart.thr().write(|w| unsafe { w.thr().bits(data) });
    }

    pub fn flush(&mut self) {
        let uart = unsafe { &*pac::UART0::PTR };
        while uart.lsr().read().thre().bit_is_clear() {}
    }

    pub fn read_until(&mut self, buf: &mut [u8], until: u8) -> Result<usize, ()> {
        let uart = unsafe { &*pac::UART0::PTR };

        // Receive FIFO Level
        let mut nread = 0;
        for i in 0..buf.len() {
            // Receive FIFO not empty
            while uart.usr().read().bits() & 0b1000 == 0 {
                core::hint::spin_loop();
            }
            // let len = uart.rfl.read() & 0b111111;

            let c = uart.thr().read().bits() as u8;
            buf[i] = c;
            nread += 1;
            if c == until {
                break;
            }
        }

        Ok(nread as usize)
    }
}

impl core::fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use core::writeln;

            writeln!(&mut $crate::uart::Uart0, $($arg)*).unwrap();
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use core::write;

            write!(&mut $crate::uart::Uart0, $($arg)*).unwrap();
        }
    }
}
