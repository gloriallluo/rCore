# lab3

计86 罗境佳 2018013469



## 实验报告

### 实验指导

-   `__switch`
    -   接受两个参数，分别是上一个任务 `current` 的 `TaskContext` 指针以及待切换的任务 `next` 的 `TaskContext`。
    -   首先将 `current` 的 `ra` 以及 `s0` 至 `s11` 等寄存器存到该任务的用户栈中，接着将 `sp` 置为 `next` 的用户栈中存 `TaskContext` 处，将 `ra` 以及 `s0-s11` 的值恢复，回到 `next.ra` 处，执行 `next` 任务。
-   启动过程
    -   `loader` 中，将用户程序一起搬运到以 `0x8040_0000` 为起始地址的内存上。
    -   每个任务的 `TrapContext` 初始化过程与第二章一致，`TaskContext` 的初始化则是将 `ra` 设为 `__restore` 的地址，再在任务对应的内核栈中开辟出对应的一片空间存 `TrapContext` 以及 `TaskContext`。
    -   之后 `run_first_task` 开始运行第一个用户程序，将第一个程序的 `TaskContext` 放入 `__switch` 中，开始运行。
-   应用切换过程
    -   从异常处理单元获得 `suspend_current_and_run_next` 或者 `exit_current_and_run_next` 的操作，将当前任务设为 `Ready` 或者 `Exited`，接着找到一个状态为 `Ready` 的任务，通过 `__switch` 切换到这个任务。



### 编程作业

#### `sys_get_time` 的实现

获得硬件执行的 cycle 数 time，并换算成 sec 以及 usec：

```rust
TimeVal {
	sec: time / CLOCK_FREQ,
	usec: (time % CLOCK_FREQ) * USEC_PER_SEC / CLOCK_FREQ
}
```



#### `stride` 算法的实现



## 问答作业

1.  **简要描述这一章的进程调度策略。何时进行进程切换？如何选择下一个运行的进程？如何处理新加入的进程？**

    触发进程切换有 3 种情况，分别是用户程序进行系统调用、发生异常以及时钟中断。其中系统调用如果是 `sys_exit` 则会停止当前的进程并切换到下一个，如果是 `sys_yield` 则会暂停当前的进程并切换到下一个。发生异常会终止当前进程并切换。发生时钟中断会暂停当前进程并切换。

    选择下一个进程在基础版本中是直接从当前的 `task_id` 开始遍历到下一个状态还是 `Ready` 的任务并开始执行，如果实现了 Stride 算法则会选择当前 `pass` 最小的任务，该算法倾向于选择执行轮数少、优先级高的进程。

    由于每个进程在初始化时其 `TaskContext::ra` 被置为 `__restore`，当该进程作为新加入的进程第一次被运行时，`__switch` 中首先将 `ra` 以及 12 个通用寄存器分别置为 `__restore` 以及 0，之后进入 `__restore` 进行一些通用寄存器的恢复（其实是将通用寄存器初始化为 0），并通过 `sret` 开始运行用户程序。

    

2.  **在 C 版代码中，同样实现了类似 RR 的调度算法，但是由于没有 VecDeque 这样直接可用的数据结构（Rust很棒对不对），C 版代码的实现严格来讲存在一定问题。大致情况如下：C版代码使用一个进程池（也就是一个 struct proc 的数组）管理进程调度，当一个时间片用尽后，选择下一个进程逻辑在 [chapter３相关代码](https://github.com/DeathWish5/ucore-Tutorial/blob/ch3/kernel/proc.c#L60-L74) ，也就是当第 i 号进程结束后，会以 i -> max_num -> 0 -> i 的顺序遍历进程池，直到找到下一个就绪进程。C 版代码新进程在调度池中的位置选择见 [chapter5相关代码](https://github.com/DeathWish5/ucore-Tutorial/blob/ch5/kernel/proc.c#L90-L98) ，也就是从头到尾遍历进程池，找到第一个空位。**

    **(2-1) 在目前这一章（chapter3）两种调度策略有实质不同吗？考虑在一个完整的 os 中，随时可能有新进程产生，这两种策略是否实质相同？**

    在目前这一章，两种调度策略实质一样，都是从当前 id 开始遍历进程池。唯一的区别是 C 版本会遍历整个进程池，其中充满未初始化的进程，而 Rust 版本则遍历到 `num_app - 1` 就会从头开始遍历。

    在有新进程产生的情况下，两者实质一样。

    

    **(2-2) 其实 C 版调度策略在公平性上存在比较大的问题，请找到一个进程产生和结束的时间序列，使得在该调度算法下发生：先创建的进程后执行的现象。你需要给出类似下面例子的信息（有更详细的分析描述更好，但尽量精简）。同时指出该序列在你实现的 stride 调度算法下顺序是怎样的？**

    线程池容量为4。“(pi, j)”表示进程 pi 在进程池的 j 号位置。

    | 时间点   | 0                             | 1           | 2           | 3           | 4       | 5       |
    | -------- | ----------------------------- | ----------- | ----------- | ----------- | ------- | ------- |
    | 运行进程 |                               | (p1, 0)     | (p2, 1)     | (p3, 2)     | (p5, 3) | (p4, 0) |
    | 事件     | (p1, 0)、(p2, 1)、(p3, 2)产生 | (p1, 0)结束 | (p4, 0)产生 | (p5, 3)产生 |         |         |

    产生顺序：p1, p2, p3, p4, p5.

    第一次执行顺序：p1, p2, p3, p5, p4.

    违反了公平性。

    在我实现的 stride 算法下：

    

3.  **stride 算法深入**

    **stride算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。**

    **实际情况是轮到 p1 执行吗？为什么？**

    在 p2 执行完后有 `p1.pass = 10, p2.pass = 4`，之后仍然是 p2 执行。这是因为仅使用 8bit 存储，做加法时发生了溢出。

    

    **我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明，在不考虑溢出的情况下，在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。**

    **为什么？尝试简单说明（传达思想即可，不要求严格证明）。**

    `STRIDE_MAX` <= `BigStride` / 2

    `STRIDE_MIN` >= `BigStride` / `isize::MAX` >= 0

    则有 `STRIDE_MAX` - `STRIDE_MIN` <= `BigStride` / 2

    

    **已知以上结论，考虑溢出的情况下，我们可以通过设计 Stride 的比较接口，结合 BinaryHeap 的 pop 接口可以很容易的找到真正最小的 Stride。**

    **请补全如下 `partial_cmp` 函数（假设永远不会相等）。**

    ```rust
    use core::cmp::Ordering;
    
    struct Stride(u64);
    
    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            // ...
        }
    }
    
    impl PartialEq for Person {
        fn eq(&self, other: &Self) -> bool {
            false
        }
    }
    ```

    例如使用 8 bits 存储 stride, BigStride = 255, 则:

    -   (125 < 255) == false
    -   (129 < 255) == true

