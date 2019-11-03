extern "C" {
    unsafe fn printf(_: &str);
}

#[direct_asm::assemble]
unsafe fn call_this(rdi: unsafe fn(&str)) {
    "
    mov rdi, eax
    call rdi
    ret
    "
}

#[test]
fn run_hello() {
    unsafe { call_this(printf) }
}
