set confirm off
set architecture riscv:rv64
target remote 127.0.0.1:1234
symbol-file /home/lqs/apue-learn/rx-os/target/riscv64gc-unknown-none-elf/debug/rx-kernel
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes
