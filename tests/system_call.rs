use libc::{c_int, c_void, size_t, ssize_t, c_long, SYS_write};

#[direct_asm::assemble]
unsafe extern "C" 
fn sys_write(fd: c_int, ptr: *const c_void, len: size_t, wcall: c_long)
    -> ssize_t 
{
    "mov rax, rcx"; // Move sys call number to rax as required
    // Other arguments are already in correct register
    "syscall"; // Invoke actual system call placed in rax
    "ret"; //Return actual result
}

fn sys_print(what: &str) -> libc::ssize_t {
    unsafe {
        sys_write(1, what.as_ptr() as *const libc::c_void, what.len(), SYS_write)
    }
}

#[test]
fn syscall() {
    // FIXME: assert we are on SysV-C-abi and UNIX
    let written = sys_print("Hello, world. Again\n\0");
    println!("Written: {}", written);
}
