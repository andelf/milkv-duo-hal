use core::marker::PhantomData;

use crate::gpio::Pull;
use crate::{into_ref, pac, peripherals, Peripheral};

pub struct Config {
    pub baudrate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { baudrate: 115200 }
    }
}

#[derive(Debug)]
pub enum Error {
    Overrun,
    Parity,
    Framing,
}

pub struct Uart<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Uart<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(_peri, tx, rx);
        T::enable_and_reset();

        tx.set_alt_function(tx.af_num());
        rx.set_alt_function(rx.af_num());

        // set pull down
        tx.set_pull(Pull::Down);
        rx.set_pull(Pull::Down);

        let uart = T::regs();

        while !uart.lsr().read().temt().bit() {
            core::hint::spin_loop();
        }

        uart.ier().write(|w| unsafe { w.bits(0x00) });
        uart.mcr().write(|w| w.dtr().set_bit().rts().set_bit()); // default
        uart.fcr()
            .write(|w| w.fifoen().set_bit().rxsr().set_bit().txsr().set_bit());

        // 8N1
        loop {
            uart.lcr()
                .modify(|_, w| unsafe { w.stb().clear_bit().pen().clear_bit().wls().bits(0b11) });
            if uart.lcr().read().stb().bit_is_clear() && uart.lcr().read().pen().bit_is_clear() {
                break;
            }
        }

        let uart_clock = 25_000_000;
        // let divisor = uart_clock / (16 * config.baudrate);
        // avoid rounding
        let divisor = (uart_clock + config.baudrate * 8) / (config.baudrate * 16);

        // Make sure LCR write wasn't ignored
        loop {
            uart.lcr().modify(|_, w| w.dlab().set_bit());
            if uart.lcr().read().dlab().bit_is_set() {
                break;
            }
        }
        uart.lpdll().write(|w| w.lpdll().variant((divisor & 0xff) as u8));
        uart.lpdlh().write(|w| w.lpdlh().variant(((divisor >> 8) & 0xff) as u8));
        uart.fcr().write(|w| w.fifoen().set_bit());

        loop {
            uart.lcr().modify(|_, w| w.dlab().clear_bit());
            if uart.lcr().read().dlab().bit_is_clear() {
                break;
            }
        }

        Self { phantom: PhantomData }
    }

    //
    fn write_lcr(&self, val: u32) {
        let mut tries = 1000;
        let uart = T::regs();
        const UART_LCR_STKP: u8 = 0x5; // 0x20
        const NON_STKP_MASK: u32 = !(1 << UART_LCR_STKP);

        while tries > 0 {
            uart.lcr().write(|w| unsafe { w.bits(val) });

            if uart.lcr().read().bits() & NON_STKP_MASK == val & NON_STKP_MASK {
                break;
            }

            // FCR_DEF_VAL
            uart.fcr()
                .write(|w| w.fifoen().set_bit().rxsr().set_bit().txsr().set_bit());
            let _ = uart.rbr().read().bits();

            tries -= 1;
        }
    }

    fn check_error(&self) -> Result<(), Error> {
        let uart = T::regs();
        let lsr = uart.lsr().read();
        if lsr.bi().bit_is_set() {
            Err(Error::Framing)
        } else if lsr.fe().bit_is_set() {
            Err(Error::Framing)
        } else if lsr.pe().bit_is_set() {
            Err(Error::Parity)
        } else if lsr.oe().bit_is_set() {
            Err(Error::Overrun)
        } else {
            Ok(())
        }
    }

    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let uart = T::regs();

        for &c in buf {
            // Wait until THR is empty
            self.check_error()?;
            // while !(uart.lsr().read().temt().bit() && uart.lsr().read().thre().bit()) {
            while !uart.usr().read().tfnf().bit() {
                core::hint::spin_loop();
            }
            uart.thr().write(|w| unsafe { w.thr().bits(c) });
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let uart = T::regs();
        while !uart.usr().read().tfe().bit() {
            core::hint::spin_loop();
        }
        Ok(())
    }

    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let uart = T::regs();

        // Receive FIFO Level
        for i in 0..buf.len() {
            // Receive FIFO not empty
            while !uart.usr().read().rfne().bit_is_set() {
                core::hint::spin_loop();
            }
            let c = uart.thr().read().bits() as u8;
            buf[i] = c;
        }
        Ok(())
    }
}

// eh

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<T> embedded_io::ErrorType for Uart<'_, T>
where
    T: Instance,
{
    type Error = Error;
}

impl<T> embedded_io::Write for Uart<'_, T>
where
    T: Instance,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()?;
        Ok(())
    }
}

pub(crate) mod sealed {
    // use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub trait Instance {
        //  type Interrupt: interrupt::Interrupt;

        fn regs() -> &'static pac::uart0::RegisterBlock;

        fn enable_and_reset();
    }
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {}

macro_rules! impl_uart {
    ($inst:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            // type Interrupt = crate::interrupt::$irq;

            fn regs() -> &'static crate::pac::uart0::RegisterBlock {
                unsafe { &*crate::pac::$inst::PTR }
            }

            fn enable_and_reset() {}
        }

        impl Instance for peripherals::$inst {}
    };
}

impl_uart!(UART0);
impl_uart!(UART1);
impl_uart!(UART2);
impl_uart!(UART3);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
pin_trait!(RtsPin, Instance);
pin_trait!(CtsPin, Instance);

pin_trait_impl!(crate::uart::TxPin, UART0, PIN_12, 0);
pin_trait_impl!(crate::uart::RxPin, UART0, PIN_13, 0);

pin_trait_impl!(crate::uart::TxPin, UART1, PIN_12, 4);
pin_trait_impl!(crate::uart::RxPin, UART1, PIN_13, 4);
pin_trait_impl!(crate::uart::RtsPin, UART1, PIN_2, 4);
pin_trait_impl!(crate::uart::CtsPin, UART1, PIN_3, 4);

pin_trait_impl!(crate::uart::TxPin, UART1, PIN_2, 6);
pin_trait_impl!(crate::uart::RxPin, UART1, PIN_3, 6);

pin_trait_impl!(crate::uart::TxPin, UART1, PIN_0, 1);
pin_trait_impl!(crate::uart::TxPin, UART2, PIN_0, 2);

pin_trait_impl!(crate::uart::RxPin, UART1, PIN_1, 1);
pin_trait_impl!(crate::uart::RxPin, UART2, PIN_1, 2);

pin_trait_impl!(crate::uart::TxPin, UART2, PIN_4, 2);
pin_trait_impl!(crate::uart::TxPin, UART3, PIN_4, 5);

pin_trait_impl!(crate::uart::RxPin, UART2, PIN_5, 2);
pin_trait_impl!(crate::uart::RxPin, UART3, PIN_5, 5);

pin_trait_impl!(crate::uart::RtsPin, UART3, PIN_8, 5);

/// Uart0 for debug, use GP12 and GP13
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
