#[direct_asm::assemble]
unsafe extern "C" fn call_this(rdi: *const u8, rsi: unsafe extern "C" fn(*const i8)) {
    "call rsi";
    "ret"
}

unsafe extern "C" fn printf(what: *const i8) {
    libc::printf(what);
}

static HELLO: &[u8] = b"Hello, world!\0";

#[test]
fn run_hello() {
    unsafe { call_this(HELLO.as_ptr(), printf) }
}
