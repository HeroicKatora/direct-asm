#[direct_asm::assemble]
unsafe fn call_this(rdi: *const u8, rsi: unsafe extern "C" fn(*const i8, ...) -> libc::c_int) {
    "call rsi
ret"
}

static HELLO: &[u8] = b"Hello, world!\0";

#[test]
fn run_hello() {
    unsafe { call_this(HELLO.as_ptr(), libc::printf) }
}
