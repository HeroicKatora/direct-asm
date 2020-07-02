#[direct_asm::assemble]
pub unsafe extern "C" fn call0(NR: usize) -> usize {
    // syscall clobbers %rcx and %r11
    // See https://stackoverflow.com/questions/47983371/why-do-x86-64-linux-system-calls-modify-rcx-and-what-does-the-value-mean/47997378#47997378
    "push %rcx";
    "push %r11";
    "syscall";
    "pop %r11";
    "pop %rcx";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call1(NR: usize, a: usize) -> usize {
    // syscall clobbers %rcx and %r11
    // See https://stackoverflow.com/questions/47983371/why-do-x86-64-linux-system-calls-modify-rcx-and-what-does-the-value-mean/47997378#47997378
    "push %rcx";
    "push %r11";
    "syscall";
    "pop %r11";
    "pop %rcx";
    "ret";
}

#[direct_asm::assemble]
pub unsafe extern "C" fn call2(NR: usize, _: usize, _: usize) -> usize {
    // syscall clobbers %rcx and %r11
    // See https://stackoverflow.com/questions/47983371/why-do-x86-64-linux-system-calls-modify-rcx-and-what-does-the-value-mean/47997378#47997378
    "push %rcx";
    "push %r11";
    "syscall";
    "pop %r11";
    "pop %rcx";
    "ret";
}
