# lab4

计86 罗境佳 2018013469



## 实验报告

### 实验指导

增加了页表机制。

增加了 MemorySet、MapArea 等数据结构，其中 MemorySet 负责 Kernel Space/一个应用程序的虚实地址转换，MapArea 则负责一片连续的 vpn 到 ppn 的转换。

### 编程作业

-   增加了 mmap 系统调用：应用程序申请内存。首先获得该应用程序的 PageTable 以及 MemorySet 等对象，接着检查即将申请的 VPN 区间是否已经分配，否则择调用 `MemorySet::insert_framed_area` 获得新的内存空间。
-   增加了 munmap 系统调用：应用程序取消一块内存的虚实映射。首先检查即将取消的VPN区间是否已经分配，若是择调用 `MapArea` 的 `unmap_one` 方法，依次取消每个 VPN 的映射，并从 `memory_set.areas` 中删除。具体可见 `MemorySet::remove_framed_area`。



## 问答作业

1.  **请列举 SV39 页表页表项的组成，结合课堂内容，描述其中的标志位有何作用／潜在作用？**

    -   addr[63:54]：reserved。
    -   addr[53:28]：PPN[2]，第一级页表的索引。
    -   addr[27:19]：PPN[1]，第二级页表的索引。
    -   addr[18:10]：PPN[0]，第三极页表的索引。
    -   addr[9:8]：RSW，保留给操作系统自行定义。
    -   addr[7]：Dirty位，记录自从上一次清零后这一页是否被修改过。
    -   addr[6]：Accessed位，记录自从上一次清零后这一页是否被访问过。
    -   addr[5]：Global Mapping位，记录是否是对所有地址空间都有效的位。
    -   addr[4]：U位，页面是否可以在U Mode下访问。
    -   addr[3]：X位，页面是否可执行，即是否可以从这一页取指。
    -   addr[2]：W位，页面是否可写。
    -   addr[1]：R位，页面是否可读。
    -   addr[0]：Valid位，这一页表项是否有效。

    

2.  **缺页**

    >   这次的实验没有涉及到缺页有点遗憾，主要是缺页难以测试，而且更多的是一种优化，不符合这次实验的核心理念，所以这里补两道小题。
    >
    >   缺页指的是进程访问页面时页面不在页表中或在页表中无效的现象，此时 MMU 将会返回一个中断，告知 os 进程内存访问出了问题。os 选择填补页表并重新执行异常指令或者杀死进程。

    -   **请问哪些异常可能是缺页导致的？**

        Load Page Fault, Store/AMO Page Fault, Instruction Page Fault.

    -   **发生缺页时，描述相关的重要寄存器值（lab2中描述过的可以简单点）。**

        stval 记录发生缺页时的地址，scause 记录异常的原因，sstatus 记录异常处理时的运行状态，stvec 记录异常处理程序的入口，sepc 记录异常发生时的 pc 值。

    >    缺页有两个常见的原因，其一是 Lazy 策略，也就是直到内存页面被访问才实际进行页表操作。比如，一个程序被执行时，进程的代码段理论上需要从磁盘加载到内存。但是 os 并不会马上这样做，而是会保存 .text 段在磁盘的位置信息，在这些代码第一次被执行时才完成从磁盘的加载操作。

    -   **这样做有哪些好处？**

        提高性能，避免没有用到的页被加载进内存；避免程序启动时花费大量时间加载所有页面；页面的使用更加灵活，提高内存的利用率。

    >    此外 COW(Copy On Write) 也是常见的容易导致缺页的 Lazy 策略，这个之后再说。其实，我们的 mmap 也可以采取 Lazy 策略，比如：一个用户进程先后申请了 10G 的内存空间，然后用了其中 1M 就直接退出了。按照现在的做法，我们显然亏大了，进行了很多没有意义的页表操作。

    -   **请问处理 10G 连续的内存页面，需要操作的页表实际大致占用多少内存（给出数量级即可）？**

        页面数量：10 * 10^9 / 4096 = 244,1406。

        一页所含的页表项数量：4096 / 8 = 512。

        页表数量：244,1406 / 512 = 4768。

        页表大小：4768 * 4096 = 1953,1248 B = 18.6 MB。

    -   **请简单思考如何才能在现有框架基础上实现 Lazy 策略，缺页时又如何处理？描述合理即可，不需要考虑实现。**

        分配页面时，并不马上建立页表项，而是仅仅在 MemorySet 中新建 MapArea 加以记录；遇到 Page Fault 时再寻找对应的 MapArea 并插入对应的页表项。

    >    缺页的另一个常见原因是 swap 策略，也就是内存页面可能被换到磁盘上了，导致对应页面失效。

    -   **此时页面失效如何表现在页表项(PTE)上？**

        页表项的 V 位被置为 0。

    

3.  **双页表与单页表**

    >    为了防范侧信道攻击，我们的 os 使用了双页表。但是传统的设计一直是单页表的，也就是说，用户线程和对应的内核线程共用同一张页表，只不过内核对应的地址只允许在内核态访问。(备注：这里的单/双的说法仅为自创的通俗说法，并无这个名词概念，详情见 [KPTI](https://en.wikipedia.org/wiki/Kernel_page-table_isolation) )

    -   **如何更换页表？**

        修改 satp 寄存器，在 `__alltraps` 中将 satp 设为 kernel token，切至内核的页表； `__restore` 中将 satp 设为 user token，切换至用户程序的页表。此外切换时需要通过 `sfence.vma` 指令清空 TLB。

    -   **单页表情况下，如何控制用户态无法访问内核页面？（tips:看看上一题最后一问）**

        将 U 位置为 0 即可。

    -   **单页表有何优势？（回答合理即可）**

        节省页表空间，一个 U 位可以代替两张页表；实现更加简单；性能更高，切换页表时不用刷新 TLB。

    -   **双页表实现下，何时需要更换页表？假设你写一个单页表操作系统，你会选择何时更换页表（回答合理即可）？**

        双页表下，一般在 S Mode 和 U Mode 切换时需要更换页表，即启动时、陷入内核时等情况下。

        假如我实现一个单页表操作系统，我会选择仅在启动时对页表进行初始化。

