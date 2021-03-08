# lab 1

计86 罗境佳 2018013469



## 实验报告

### 本次编程内容

-   实验指导书

    实现了一个能够建立栈空间、清除 `.bss` 段、调用 `rust_sbi` 实现输入/输出/关机并且运行在裸机上的简易 os。

-   编程作业

    利用 ANSI 转义序列实现了命令行的彩色输出。分别实现了 `error!`、`warn!`、`debug!`、`info!`、`trace!` 等 5 个宏，以显示不同的信息。由于 5 个宏的实现又有许多重复的地方，因此在 `console.rs` 中定义了 `impl_color_print!` 宏来减少重复代码。

### 实验效果

<img src="./images/color_console.png" alt="color_console" style="zoom:67%;" />



## 问答作业

1.  为了方便 os 处理，Ｍ态软件会将 S 态异常/中断委托给 S 态软件，请指出有哪些寄存器记录了委托信息，rustsbi 委托了哪些异常/中断？（也可以直接给出寄存器的值）

    记录委托信息的寄存器：`x10`、`x11`、`x12`、`x17`。

    rustsbi 记录委托信息的寄存器包括  `mideleg`、`medeleg`、`mie`。委托的异常/中断包括：

    ```rust
    unsafe {
    	mideleg::set_sext();
        mideleg::set_stimer();
    	mideleg::set_ssoft();
        medeleg::set_instruction_misaligned();
        medeleg::set_breakpoint();
        medeleg::set_user_env_call();
        medeleg::set_instruction_page_fault();
        medeleg::set_load_page_fault();
        medeleg::set_store_page_fault();
        medeleg::set_instruction_fault();
        medeleg::set_load_fault();
        medeleg::set_store_fault();
        mie::set_mext();
        // 不打开mie::set_mtimer
        mie::set_msoft();
    }
    ```

    

2.  请学习 gdb 调试工具的使用（这对后续调试很重要），并通过 gdb 简单跟踪从机器加电到跳转到 0x80200000 的简单过程。只需要描述重要的跳转即可，只需要描述在 qemu 上的情况。

    -   **qemu**
        -   `pc` 初始值为 0x1000，将该值存入 `t0` 寄存器。
        -   给 `a0` 赋值为 `mhartid`，给 `a1` 赋值为 0x1020。
        -   跳转到 `Mem[0x1018]`，具体的值为 0x8000_0000，之后便进入 rust-sbi 的代码部分。
    -   **rust-sbi**
        -   将 `_max_hart_id` 读入 `t0`，如果小于 `mhartid` 则进入 `_start_abort`。
        -   令`sp` 等于 _stack\_start - _hart_stack_size * mhartid。
        -   令 `mscratch` 等于 0。
        -   进入 `main`，进行一些 rust-sbi 的初始化，将 S 的中断委托给 S 态。
        -   从 M 态进入 S 态，运行操作系统。

