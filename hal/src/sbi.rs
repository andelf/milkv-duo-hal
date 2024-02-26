#![allow(unused)]

use core::arch::asm;

const EID_BASE: usize = 0x10;

#[derive(Debug)]
pub struct SbiRet {
    pub error: usize,
    pub value: usize,
}

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiRet {
    let error: usize;
    let value: usize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    SbiRet { error, value }
}

pub fn get_sbi_specification_version() -> SbiRet {
    const FID: usize = 0;
    sbi_call(EID_BASE, FID, 0, 0, 0)
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

#[inline(always)]
fn sbi_call_legacy(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") which,
        );
    }
    ret
}

pub fn console_putchar(ch: u8) {
    sbi_call_legacy(SBI_CONSOLE_PUTCHAR, ch as usize, 0, 0);
}

pub fn console_getchar() -> u8 {
    sbi_call_legacy(SBI_CONSOLE_GETCHAR, 0, 0, 0) as u8
}

pub fn set_timer(stime_value: u64) {
    sbi_call_legacy(SBI_SET_TIMER, stime_value as usize, 0, 0);
}

pub fn put_string(s: &str) {
    for ch in s.bytes() {
        console_putchar(ch);
    }
}
