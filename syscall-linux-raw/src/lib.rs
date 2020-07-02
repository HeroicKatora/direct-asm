//! # Tables of possibly supported platforms.
//!
//! ## Syscall and ret instructions and registers 
//!
//! ```
//! Arch/ABI    Instruction           System# Ret1  Ret2
//! arm/OABI    swi NR                -       r0    -
//! arm/EABI    swi 0x0               r7      r0   r1
//! arm64       svc #0                w8      x0   x1
//! i386        int $0x80             eax     eax  edx
//! riscv       ecall                 a7      a0   a1
//! x86-64      syscall               rax     rax  rdx
//! x32         syscall               rax     rax  rdx
//! ```
//!
//! ## Argument registers
//!
//! ```
//! Arch/ABI      arg1  arg2  arg3  arg4  arg5  arg6  arg7  Notes
//! arm/OABI      r0    r1    r2    r3    r4    r5    r6
//! arm/EABI      r0    r1    r2    r3    r4    r5    r6
//! arm64         x0    x1    x2    x3    x4    x5    -
//! i386          ebx   ecx   edx   esi   edi   ebp   -
//! riscv         a0    a1    a2    a3    a4    a5    -
//! x86-64        rdi   rsi   rdx   r10   r8    r9    -
//! x32           rdi   rsi   rdx   r10   r8    r9    -
//! ```
//!
//! ## C-abi registers
//!
//! ```
//! Arch/ABI      a0    a1    a2    a3    a4    a5    ret1  ret2  caller
//! x86-64        rdi   rsi   rdx   rcx   r8    r9    rax   rdx   rbx,rsp,rbp
//! ```
// Prepare for the future, we might want to justify our inner translation.
#![allow(unused_unsafe)]
#![no_std]

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod impl_;

/* These are not yet supported by the assembler. Compiling them would yield invalid
 * instructions on their platforms.
#[cfg(target_arch = "x86")]
#[path = "x86.rs"]
mod impl_;

#[cfg(target_arch = "aarch64")]
#[path = "aarch64.rs"]
mod impl_;
*/


/// A helper to distinguish the syscall number from other parameters.
///
/// We can't really help you with others but this one is somewhat important. Furthermore it's a
/// great help internally where it ensure we've placed the parameter at the right location. The
/// type is a transparent wrapper.
///
/// This further enables a namespaced access to constants and call numbers that are conditionally
/// available per system ABI.
#[repr(transparent)]
pub struct SysNr(pub isize);

#[cfg(target_os = "linux")]
pub unsafe fn syscall0(nr: SysNr) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call0(nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall1(nr: SysNr, a: isize) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call1(a, nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall2(nr: SysNr, a: isize, b: isize) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call2(a, b, nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall3(nr: SysNr, a: isize, b: isize, c: isize) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call3(a, b, c, nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall4(nr: SysNr, a: isize, b: isize, c: isize, d: isize) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call4(a, b, c, d, nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall5(nr: SysNr, a: isize, b: isize, c: isize, d: isize, e: isize) -> isize {
    // Translate to the inner abi
    unsafe { impl_::call5(a, b, c, d, e, nr) }
}
