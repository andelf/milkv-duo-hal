OUTPUT_ARCH(riscv)
ENTRY(_start)
MEMORY {
    /* reused TPU_SRAM, 220KB */
    SRAM : ORIGIN = 0x0C000000, LENGTH = 0x37000
}
SECTIONS {
    .text : ALIGN(8) {
        *(.text.entry)
        *(.text .text.*)
    } > SRAM
    .rodata : ALIGN(8) {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(8);
        erodata = .;
    } > SRAM
    .data : ALIGN(8) {
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(8);
        edata = .;
    } > SRAM
    sidata = LOADADDR(.data);
    .bss (NOLOAD) : ALIGN(8) {
        *(.bss.uninit)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss = .;
    } > SRAM
    /DISCARD/ : {
        *(.eh_frame)
    }

    /* rom api */
    p_rom_api_cryptodma_aes_decrypt = 0x0000000004418100;
    p_rom_api_flash_init = 0x0000000004418080;
    p_rom_api_get_boot_src = 0x0000000004418020;
    p_rom_api_get_number_of_retries = 0x00000000044180c0;
    p_rom_api_image_crc = 0x00000000044180a0;
    p_rom_api_load_image = 0x0000000004418060;
    p_rom_api_set_boot_src = 0x0000000004418040;
    p_rom_api_verify_rsa = 0x00000000044180e0;
}
