# lab2

计86 罗境佳 2018013469



## 实验报告

-   实验指导

    本次 lab 主要完成了一个顺序执行用户程序的批处理程序，以及一个 `Trap` 处理程序。

-   编程作业



## 问答作业

1.  正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。目前由于一些其他原因，这些问题不太好测试，请同学们可以自行测试这些内容（参考 [前三个测例](https://github.com/DeathWish5/rCore_tutorial_tests/tree/master/user/src/bin) )，描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
2.  请结合用例理解 [trap.S](https://github.com/rcore-os/rCore-Tutorial-v3/blob/ch2/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下几个问题：
3.  描述程序陷入内核的两大原因是中断和异常，请问 riscv64 支持哪些中断／异常？如何判断进入内核是由于中断还是异常？描述陷入内核时的几个重要寄存器及其值。
4.  对于任何中断， `__alltraps` 中都需要保存所有寄存器吗？你有没有想到一些加速 `__alltraps` 的方法？简单描述你的想法。