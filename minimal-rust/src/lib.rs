//! Demonstrates basic assembling for a full program.
#![crate_type="rlib"]

fn exit() -> ! {
    #[direct_asm::assemble]
    unsafe extern "C" fn exit_raw() -> ! {
        "xor %rdi, %rdi";
        "mov %rax, 60";
        // Argument setup in edi
        "syscall"
    }
    unsafe {
        exit_raw()
    }
}

fn write(fd: usize, buf: &[u8]) {
    #[direct_asm::assemble]
    unsafe extern "C" fn write_raw(fd: usize, buf: *const u8, len: usize) -> isize {
        "mov %rax, 1";
        "syscall";
        "ret"
    }

    unsafe {
        write_raw(fd, buf.as_ptr(), buf.len());
    }
}

#[no_mangle]
pub fn main() {
    write(1, "Hello!\n".as_bytes());
    exit();
}
