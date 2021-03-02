use super::SysNr;

#[direct_asm::assemble]
pub unsafe extern "C" fn call0(NR: SysNr) -> isize {
    "mov %rax,%rdi";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call1(a: isize, NR: SysNr) -> isize {
    "mov %rax,%rsi";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call2(_: isize, _: isize, NR: SysNr) -> isize {
    "mov %rax,%rdx";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call3(_: isize, _: isize, _: isize, NR: SysNr) -> isize {
    "mov %rax,%rcx";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call4(_: isize, _: isize, _: isize, _: isize, NR: SysNr) -> isize {
    // syscall clobbers %rcx and %r11
    // See https://stackoverflow.com/questions/47983371/why-do-x86-64-linux-system-calls-modify-rcx-and-what-does-the-value-mean/47997378#47997378
    "mov %r10,%rcx";
    "mov %rax,%r8";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call5(_: isize, _: isize, _: isize, _: isize, _: isize, NR: SysNr) -> isize {
    "mov %r10,%rcx";
    "mov %rax,%r9";
    "syscall";
    "ret";
}

// TODO: figure out call6
