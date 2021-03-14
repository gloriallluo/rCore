# Lab 2

计86 罗境佳 2018013469



## 实验报告

### 实验指导

本次实验完成了一个批处理程序，并且能够简单地处理用户程序产生的异常。

-   启动过程
    -   将 `__alltraps` 写入 `stvec`。
    -   加载第一个程序。
-   应用切换过程
    -   在启动时 / 应用遇到无法解决的错误时 / 应用自然运行结束时，会进行这一套流程。
    -   `run_next_app`：
        -   `load_app`：将原来程序的代码清空，将新程序的代码加载进 `0x8040_0000` 处。
        -   `app_init_context`：为新的用户程序生成新的 `TrapContext`，其中包括将 `ssatus.SPP` 设为 U-Mode，将通用寄存器置零，将 `sepc` 置为 `APP_BASE_ADDRESS` 以便之后给 `pc` 赋值，将 `sp` 置为用户栈的栈底。
        -   `__restore`：起初运行在 S-Mode，恢复 U-Mode 运行时的现场，最后通过 `sret` 回到 U-Mode。
-   处理异常过程
    -   `__alltraps`：运行在 S-Mode，保存用户程序的执行现场，最后进入 Rust 函数 `trap_handler`。
    -   `trap_handler`：根据 `scause` 进入不同的异常处理流程。目前实现的主要有，在用户程序进行系统调用时，进入 `syscall` 函数，在发生 `PageFault` 或者是非法指令异常时，直接运行下一个程序。

### 编程作业

需要在用户调用 `sys_write` 这一系统调用时，检查待输出的内容是否位于该应用的合法地址内。合法地址包括：

-   用户代码：

    ````rust
    APP_BASE_ADDRESS..APP_BASE_ADDRESS + self.app_start[self.current_app] - self.app_start[self.current_app - 1]
    ````

-   用户栈：

    ```rust
    cx.x[2]..USER_STACK.get_sp() // 从用户 syscall 时的 sp 到用户栈的栈顶
    ```

检查 `buf..buf+len` 是否位于这两个区间内即可。



## 问答作业

1.  正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。目前由于一些其他原因，这些问题不太好测试，请同学们可以自行测试这些内容（参考 [前三个测例](https://github.com/DeathWish5/rCore_tutorial_tests/tree/master/user/src/bin) )，描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

    sbi 及版本：`[rustsbi] RustSBI version 0.1.1`

    -   非法指令 `sret`

        ```shell
        [rustsbi-panic] hart 0 panicked at 'invalid instruction, mepc: 000000008040005c, instruction: 0000000010200073', platform/qemu/src/main.rs:458:17
        ```

        在用户程序运行前，操作系统直接退出。

    -   非法访问寄存器 `csrr $0, sstatus`

        ```shell
        [rustsbi-panic] hart 0 panicked at 'invalid instruction, mepc: 000000008040005e, instruction: 0000000010002573', platform/qemu/src/main.rs:458:17
        ```

        在用户程序运行前，操作系统直接退出。

    -   非法地址 `(0x0 as *mut u8).write_volatile(0);`

        ```shell
        [kernel] PageFault in application, core dumped.
        ```
        
        在用户程序运行时，报出 `PageFault` 的异常，用户程序退出，运行下一个用户程序。

    

2.  请结合用例理解 [trap.S](https://github.com/rcore-os/rCore-Tutorial-v3/blob/ch2/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下几个问题:

    `__alltraps`：运行在 S-Mode，保存用户程序的执行现场，最后进入 Rust 函数 `trap_handler`。

    `__restore`：起初运行在 S-Mode，恢复 U-Mode 运行时的现场，最后通过 `sret` 回到 U-Mode。

    1.  L40: 刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

        调用 `__restore` 传入了一个 `TrapContext` 的地址作为参数，`a0` 指向这个 `TrapContext`。

        两种使用情景分别是在机器加电启动时以及切换至下一个应用时。

        

    2.  L46-L51: 这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

        特殊处理的寄存器及意义包括：

        -   `cx sstatus -> t0 -> real sstatus`：`__restore` 在处理完用户程序的异常之后调用，由于在处理异常过程中可能对 `sstatus` 进行修改，因此要从栈中找出异常处理前的 `sstatus` 值存进 `sstatus` 寄存器。
        -   `cx sepc -> t1 -> real sepc`：将 `sepc` 从栈中取出，恢复用户程序的 `sepc` 寄存器，异常处理过程中可能会发生其他的异常，进而 `sepc` 被修改，因此需要在异常处理完成后恢复。
        -   `user sp -> t2 -> sscratch`：在异常处理程序时，用 `sscratch` 来保存 `sp`，在返回时将 `sp` 恢复，重新指向用户栈。

        

    3.  L53-L59: 为何跳过了 `x2` 和 `x4`？

        ```
        ld x1, 1*8(sp)
        ld x3, 3*8(sp)
        .set n, 5
        .rept 27
           LOAD_GP %n
           .set n, n+1
        .endr
        ```

        `x2` 是 `sp`，`x4` 是 `tp`。跳过 `x2` 是因为在异常处理过程中一般在 `sscratch` 中保存 `sp` 而非在栈上，且此时正在从栈中恢复通用寄存器，不方便修改栈顶；跳过 `x4` 是因为整个过程中并没有修改/使用它，因此也没有存在栈上。

        

    4.  L63: 该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

        ```
        csrrw sp, sscratch, sp
        ```

        这条指令交换 `sp` 和 `sscratch` 中的值。在这条指令之后，`sp` 指向用户栈，而 `sscratch` 指向内核栈，即将返回用户程序。

        

    5.  `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

        是在 L64 的 `sret` 指令，该指令使计算机从 S-Mode 回到 U-Mode，并从 `sepc` 开始执行。

        

    6.  L13： 该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

        ```
        csrrw sp, sscratch, sp
        ```

        在这条指令之后，`sp` 指向内核栈，` sscratch` 指向用户栈，即将开始处理异常。

        

    7.  从 U 态进入 S 态是哪一条指令发生的？

        是在 `__alltraps` 前进入 S 态的。因为 CPU 遇到用户程序的异常时，在 CSR 寄存器中保存异常信息，之后进入 `stvec` 中所保存的异常处理程序入口。且 `__alltraps` 的第一句（L13）已经可以用代码显式对 S 态 CSR 寄存器进行修改，说明此时已在 S 态。

    

3.  程序陷入内核的原因有中断和异常（系统调用），请问 riscv64 支持哪些中断 / 异常？如何判断进入内核是由于中断还是异常？描述陷入内核时的几个重要寄存器及其值。

    riscv64 支持的中断：Software Interrupt, Timer Interrupt, External Interrupt.

    riscv64 支持的异常：Instruction Address Misaligned, Instruction Access Fault, Illegal Instruction, Breakpoint, Load Address Misaligned, Load Access Fault, Store/AMO Address Misaligned, Store/AMO Access Fault, Environment Call From U-mode, Environment Call From S-mode, Instruction/Load/Store Page Fault.

    判断中断和异常是根据 `scause[SXLEN-1]` 来判断，若是 0 则是异常，若是 1 则是中断。

    陷入内核时重要的寄存器除了 `scause` 以外，还有 `sepc` 来存储旧的 PC 值，`stvec` 来存储异常处理程序的地址，`sstatus` 来存储异常信息等等。

    

4.  对于任何中断， `__alltraps` 中都需要保存所有寄存器吗？你有没有想到一些加速 `__alltraps` 的方法？简单描述你的想法。

    `__alltraps` 中保存的寄存器包括除 `x0`、`x2`、`x4` 以外的其他 29 个通用寄存器，`sstatus`、`sepc` 以及 `sscratch`。对于程序产生的 Exception（如页错误、错误指令等等），目前操作系统对其往往是不作处理直接进入下一个用户程序。因此在 `__alltraps` 内可以先判断一下 `scause[63]` 是否为 0 且不是系统调用/断点，这样可以不存储通用寄存器以及 `sepc`。

