set confirm off
set architecture riscv:rv64
target remote 127.0.0.1:1234
symbol-file /home/lqs/apue-learn/rx-os/target/riscv64gc-unknown-none-elf/debug/rx-kernel
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes

# set $lockn = 0
# set $unlockn = 0

# b src/lock/mutex.rs:43
# b src/lock/mutex.rs:79
# commands 1 
#     set $lockn = $lockn + 1
#     p self.name
#     p self
#     c
# end
# commands 2
#     set $unlockn = $unlockn + 1
#     p self.name
#     p self
#     c
# end

# b src/process/cpu.rs:53

# commands 3
#     p self
#     if self.noff==1
#         c 
#     else
#         p $lockn
#         p $unlockn
#     end
# end


# c