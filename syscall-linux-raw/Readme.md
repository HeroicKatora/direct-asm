# syscall-linux-raw

Defines syscallN methods for performing raw Linux syscalls.

## What

Defines the small set of methods abstracting raw syscalls with different
numbers of arguments, only available on `target_os = "linux"`. That's it. But
it works on stable and may be instructive for other syscall binary interfaces.

## How

By pure magic. Jk, it defines fully assembled binary function with a proper
C-abi and aliases them as callable functions with `#[no_mangle]`, then defines
a bunch of wrappers with Rust abi that should at some point be available
independent of the ISA chosen in the backend.

## Wtf

Indeed. Don't use in prod.

## Demo?

See the example: `cargo run --example simple`.

Only works on `x86_64-unknown-linux-*` and should fail to compile on other
architectures and OS's. It also works on `x86_64-unknown-none` but all methods
are unsafe and the contract of calling a Linux OS must be upheld by the caller.

## License

The base software: [Unlicense](https://unlicense.org/)
