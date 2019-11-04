# direct-asm

A Rust proc-macro to include pre-assembled instructions as a function call.

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

## License

[Unlicense](https://unlicense.org/)
