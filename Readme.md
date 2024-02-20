# direct-asm

A Rust proc-macro to include pre-assembled instructions as a function call.

## What

```rust
use libc::{c_int, c_void, size_t, ssize_t, c_long};

// Call write as a syscall on SysV x86-64 abi.
// WIP: The constant SYS_write needs to be provided by the caller for now.
#[direct_asm::assemble]
unsafe extern "C" 
fn sys_write(fd: c_int, ptr: *const c_void, len: size_t, wcall: c_long)
    -> ssize_t 
{
    "mov rax, rcx"; // Move sys call number to rax as required
    // Other arguments are already in correct register
    "syscall"; // Invoke actual system call placed in rax
    "ret" //Return actual result
}

fn sys_print(what: &str) -> libc::ssize_t {
    unsafe {
        sys_write(1, what.as_ptr() as *const libc::c_void, what.len(), SYS_write)
    }
}
```

## Why

To show an alternative to `inline-asm` from gcc, possibly with more control
while having well defined semantics. This will not be sufficient for all
purposes but it is enough to read stack registers, to make system calls (I
think) and much more. The included code must be position independent, has no
access to globals (pass as arguments instead), and can not introduce any new
symbols.

## How

By aliasing two definitions with `#[no_mangle]` abuse. We precompile the asm
using `nasm` into a raw binary form, then define a static byte array containing
this code in the `.text` section and finally define an `extern "C"` function
with the same symbol name. The linker then resolve that function to the array
definition and hence calls the code as intended.

## Wtf

Indeed. Don't use in prod.

## Demo?

Within the `minimal-rust` folder we build a 186-byte binary on stable Rust.

Also, the `syscall-linux-raw` folder demonstrates a 936-byte "Hello, world!"
binary on decently ergonomic stable Rust. And we even properly call `exit(0)`.

## License

The base software: [Unlicense](https://unlicense.org/)

The modified `dynasm` backend: Mozilla Public License Version 2.0
