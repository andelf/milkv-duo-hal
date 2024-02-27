#![macro_use]

macro_rules! pin_trait {
    ($signal:ident, $instance:path) => {
        pub trait $signal<T: $instance>: crate::gpio::Pin {
            fn af_num(&self) -> u8;
        }
    };
}
macro_rules! pin_trait_impl {
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident, $af_num:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance> for crate::peripherals::$pin {
            #[inline(always)]
            fn af_num(&self) -> u8 {
                $af_num
            }
        }
    };
}
