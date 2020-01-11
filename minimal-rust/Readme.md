Demonstrate the usefulness of direct-asm to build minimal binaries on stable
Rust. This only works on `x86_64-linux` since it is obviously dependent on
loader and the system call interface. However, it should be easy to swap the
actual function definitions depending on some `cfg` macros that check targets
and support other OS's that can execute ELF. Pull requests are welcome.

The file `src/lib.rs` is published under the terms of [Unlicense](./UNLICENSE).

The files `script.ld`, `elf.s`, and `build.sh` have been take from another
minimal binary project and adapted to a more recent version using `cargo` as
well. They have been used as allowed under the following conditions:

> This project is copyright 2015, Keegan McAllister <kmcallister@mozilla.com>
> 
> Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
> http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
> http://opensource.org/licenses/MIT>, at your option. All files in the project
> carrying such notice may not be copied, modified, or distributed except
> according to those terms.
