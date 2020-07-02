use super::SysNr;

#[direct_asm::assemble]
pub unsafe extern "C" fn call0(NR: SysNr) -> isize {
    "mov %rdi,%rax";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call1(a: isize, NR: SysNr) -> isize {
    "mov %rsi,%rax";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call2(_: isize, _: isize, NR: SysNr) -> isize {
    "mov %rcx,%rax";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call3(_: isize, _: isize, _: isize, NR: SysNr) -> isize {
    "mov %r8,%rax";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call4(_: isize, _: isize, _: isize, _: isize, NR: SysNr) -> isize {
    // syscall clobbers %rcx and %r11
    // See https://stackoverflow.com/questions/47983371/why-do-x86-64-linux-system-calls-modify-rcx-and-what-does-the-value-mean/47997378#47997378
    "mov %rcx,%r10";
    "mov %r8,%rax";
    "syscall";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call5(_: isize, _: isize, _: isize, _: isize, _: isize, NR: SysNr) -> isize {
    "mov %rcx,%r10";
    "mov %r9,%rax";
    "syscall";
    "ret";
}

// TODO: figure out call6
