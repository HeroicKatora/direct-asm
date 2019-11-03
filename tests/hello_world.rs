extern "C" {
    fn printf(_: *const u8);
}

#[direct_asm::assemble]
unsafe fn call_this(rdi: *const u8, rsi: unsafe extern "C" fn(*const u8)) {
    "
    mov rdi, eax
    call rsi
    ret
    "
}

static HELLO: &[u8] = b"Hello, world!\0";

#[test]
fn run_hello() {
    unsafe { call_this(HELLO.as_ptr(), printf) }
}
