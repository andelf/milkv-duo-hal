#![no_std]
#![feature(naked_functions, asm_const)]

use core::arch::asm;
const LEN_STACK: usize = 1 * 1024 * 1024;
const STACK_START: usize = 0x80000000 + 240 * 1024 * 1024;

pub use riscv_rt_macros::entry;

#[repr(align(16))]
pub struct Stack<const N: usize>([u8; N]);

#[naked]
#[link_section = ".text.entry"]
#[export_name = "_start"]
unsafe extern "C" fn entry() -> ! {
    //#[link_section = ".bss.uninit"]
    //static mut STACK: Stack<LEN_STACK> = Stack([0; LEN_STACK]);

    asm!(
        ".option push
        .option arch, -c
            j       1f
        .option pop",          // Fake header for OpenSBI
        ".word   0x33334c42",  // BL33
        ".word   0xdeadbeea",  // BL2 MSID
        ".word   0xdeadbeeb",  // BL2 version
        ".word   0x80200000",  // Load address
        ".word   0",
        ".word   0xdeadbeec",
        ".word   0x0001a011",
        "1:",
        // configure mxstatus register
        // PM = 0b11 (Current privilege mode is Machine mode)
        // THEADISAEE = 1 (Enable T-Head ISA)
        // MAEE = 1 (Enable extended MMU attributes)
        // MHRD = 0 (Disable TLB hardware refill)
        // CLINTEE = 1 (CLINT usoft and utimer can be responded)
        // UCME = 1 (Enable extended cache instructions on U-mode)
        // MM = 1 (Enable hardware unaligned memory access)
        // PMP4K = 0 (read-only, PMP granularity 4KiB)
        // PMDM = 0 (allow performance counter on M-mode)
        // PMDS = 0 (allow performance counter on S-mode)
        // PMDU = 0 (allow performance counter on U-mode)
        //"   li      t0, 0xc0638000
        //    csrw    0x7c0, t0",
        // invalid I-cache, D-cache, BHT and BTB by writing mcor register
        //"   li      t2, 0x30013
        //    csrw    0x7c2, t2",
        // enable I-cache, D-cache by mhcr register
        //"   csrsi   0x7c1, 0x3",
        // load stack address
        "   la      sp, {stack}
            li      t0, {hart_stack_size}
            add     sp, sp, t0",
        // clear bss segment
        "	la  	t1, sbss
        	la   	t2, ebss
    	1:  bgeu 	t1, t2, 1f
        	sd   	zero, 0(t1)
        	addi 	t1, t1, 8
        	j    	1b
    	1:",
        "   call    {main}",
        stack = const STACK_START,
        hart_stack_size = const LEN_STACK,
        main = sym main,
        options(noreturn)
    )
}

extern "Rust" {
    fn main() -> !;
}
