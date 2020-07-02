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

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod impl_;

#[cfg(target_os = "linux")]
pub unsafe fn syscall0(nr: usize) -> usize {
    /* These are not yet supported by the assembler. Compiling them would yield invalid
     * instructions on their platforms.
    #[cfg(target_arch = "x86")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }

    #[cfg(target_arch = "aarch64")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }

    #[cfg(target_arch = "aarch64")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }
    */

    // Translate to the inner abi
    unsafe { impl_::call0(nr) }
}

#[cfg(target_os = "linux")]
pub unsafe fn syscall1(nr: usize, a: usize) -> usize {
    /* These are not yet supported by the assembler. Compiling them would yield invalid
     * instructions on their platforms.
    #[cfg(target_arch = "x86")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }

    #[cfg(target_arch = "aarch64")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }

    #[cfg(target_arch = "aarch64")]
    #[direct_asm::assemble]
    unsafe extern "C" fn call1_inner(NR: usize) -> usize {
        "ret"
    }
    */

    // Translate to the inner abi
    unsafe { impl_::call1(nr, a) }
}
