//! Demonstrates basic assembling for a full program.
#![no_std]
#![no_main]

fn exit() -> ! {
    unsafe {
        syscall_linux_raw::syscall1(syscall_linux_raw::SysNr(60), 0);
        core::hint::unreachable_unchecked();
    }
}

fn write(fd: usize, buf: &[u8]) {
    unsafe {
        syscall_linux_raw::syscall3(syscall_linux_raw::SysNr(1), fd as isize, buf.as_ptr() as isize, buf.len() as isize);
    }
}

#[no_mangle]
pub extern fn main() {
    write(1, "Hello, world!\n".as_bytes());
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle] fn __libc_csu_fini() {}
#[no_mangle] fn __libc_csu_init() {}
#[no_mangle] fn __libc_start_main() -> ! { main(); exit() }
