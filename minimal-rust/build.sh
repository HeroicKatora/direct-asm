cargo rustc --release -- \
    -O -C no-stack-check -C relocation-model=static

rlib=$(ar t target/release/libminimal_rust.rlib | grep "\\.o$")
echo $rlib
ar x target/release/libminimal_rust.rlib "$rlib"

objdump -dr "$rlib"
echo

ld --gc-sections -e main -T script.ld -o payload "$rlib"
objcopy -j combined -O binary payload payload.bin

ENTRY=$(nm -f posix payload | grep '^main ' | awk '{print $3}')
nasm -f bin -o direct-asm-tiny -D entry=0x$ENTRY elf.s

echo "Final size:" $(wc -c direct-asm-tiny)
