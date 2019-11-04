use libc::SYS_write;

#[direct_asm::assemble]
unsafe fn sys_write(fd: libc::c_int, ptr: *const libc::c_void, len: libc::size_t, wcall: libc::c_long) -> libc::ssize_t {
    // Call styles differ: Move syscall from 4th SysV arg to rax
    "mov rax, rcx
    syscall
    ret"
}

fn sys_write_stdout(what: &str) -> libc::ssize_t {
    unsafe {
        sys_write(1, what.as_ptr() as *const libc::c_void, what.len(), SYS_write)
    }
}

#[test]
fn syscall() {
    let written = sys_write_stdout("Hello, world. Again\n\0");
    println!("Written: {}", written);
}
