use core::mem;
use core::arch::asm;

#[inline(always)]
pub fn sin(a: f32) -> f32 {
    a
    //
    // let mut res: f32 = unsafe { mem::uninitialized() };
    //
    // unsafe { asm!(
    //     r##"
    //         flds $1;
    //         fsin;
    //         fstps $0;
    //     "##
    //     : "=*m"(&mut res as *mut f32)
    //     : "*m"(&a as *const f32)
    // ) };
    //
    // res
}

#[inline(always)]
pub fn cos(a: f32) -> f32 {
    a
    //
    // let mut res: f32 = unsafe { mem::uninitialized() };
    //
    // unsafe { asm!(
    //     r##"
    //         flds $1;
    //         fcos;
    //         fstps $0;
    //     "##
    //     : "=*m"(&mut res as *mut f32)
    //     : "*m"(&a as *const f32)
    // ) };
    //
    // res
}
