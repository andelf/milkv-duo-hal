use crate::pac;

// We need to export this in the hal for the drivers to use

crate::peripherals! {
    UART0 <= UART0,
    UART1 <= UART1,
    UART2 <= UART2,
    UART3 <= UART3,
    UART4 <= UART4,


    PIN_0 <= virtual,
    PIN_1 <= virtual,
    PIN_2 <= virtual,
    PIN_3 <= virtual,
    PIN_4 <= virtual,
    PIN_5 <= virtual,
    PIN_6 <= virtual,
    PIN_7 <= virtual,
    PIN_8 <= virtual,
    PIN_9 <= virtual,
    PIN_10 <= virtual,
    PIN_11 <= virtual,
    PIN_12 <= virtual,
    PIN_13 <= virtual,
    PIN_14 <= virtual,
    PIN_15 <= virtual,
    PIN_16 <= virtual,
    PIN_17 <= virtual,
    PIN_18 <= virtual,
    PIN_19 <= virtual,
    PIN_20 <= virtual,
    PIN_21 <= virtual,
    PIN_22 <= virtual,

    PIN_26 <= virtual, // ADC1
    PIN_27 <= virtual, // ADC2

    PIN_LED <= virtual,

    PIN_RUN <= virtual,

    PIN_SD0_CD <= virtual,
    PIN_SD0_D0 <= virtual,
    PIN_SD0_D1 <= virtual,
    PIN_SD0_D2 <= virtual,
    PIN_SD0_D3 <= virtual,
    PIN_SD0_CMD <= virtual,
    PIN_SD0_CLK <= virtual,



}
