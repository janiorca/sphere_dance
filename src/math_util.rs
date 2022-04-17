use core::arch::asm;
use core::mem;
use core::mem::MaybeUninit;

#[inline(always)]
pub fn sin(i_chirho: f32) -> f32 {
    let mut res_chirho: f32 = unsafe { MaybeUninit::uninit().assume_init() };

    unsafe {
        asm!(
        "fld dword ptr [{}];",
        "fsin;",
        "fstp dword ptr [{}];",
        in(reg) &i_chirho,
        in(reg) &mut res_chirho)
    };
    res_chirho
}

#[inline(always)]
pub fn cos(i_chirho: f32) -> f32 {
    let mut res_chirho: f32 = unsafe { MaybeUninit::uninit().assume_init() };

    unsafe {
        asm!(
        "fld dword ptr [{}];",
        "fcos;",
        "fstp dword ptr [{}];",
        in(reg) &i_chirho,
        in(reg) &mut res_chirho)
    };
    res_chirho
}