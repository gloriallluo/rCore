# lab 6

计86 罗境佳 2018013469



## 实验报告

### 实验指导

本节实验实现了文件的读写以及通过管道的进程间通信。在每个进程控制块中增加了一个 `fd_table`，为一个打开文件表，其中放入实现了 `File + Send + Sync` 接口的文件抽象数据结构。

-   文件读写：需要实现 `File` 接口，其中包括 `read` 和 `write` 两个函数，分别是将文件内容读入用 `UserBuffer` 封装的一片内存中、以及将内存中的内容写入 `UserBuffer`。
-   文件打开/关闭：打开文件会在进程的 `fd_table` 中新分配一个文件描述符的位置，关闭则是将对应 `fd` 在 `fd_table` 中置为 `None`，之后该进程不能对该文件进行读写。
-   管道：同样被抽象为文件，具体的实现是通过一个循环队列，应该位于创建该 `Pipe` 的进程的内核栈中。通过 `make_pipe` 函数新建一个管道，将管道的读端和写端放入两个进程（只支持自己和自己/父子进程间进行通信）的 `fd_table` 中。

### 编程作业

需要实现进程间的 UDP 邮件通信。本着“一切皆文件”的思想，也将邮件抽象成了文件，具体实现请见 `crate::task::mail`。在进程控制块内加了一个邮件队列，表示进程的“收件箱”，具体为 `mail_box: VecDeque<Arc<Mail>>`。写邮件则会在一个新建的 `Mail` 中写入内存中的内容，并放入 `mail_box` 的末尾；读邮件则会从 `mail_box` 的队首 pop 出一个并读入内存。



## 问答作业

1.  **举出使用 pipe 的一个实际应用的例子。**

    ```shell
    cat names.txt | grep -n -c "XiaoMing"
    ```

    将第一个命令的输出从管道传输到第二个命令作为输入。

    `cat` 命令将文件中的内容逐行输出（默认是终端），之后第二条命令会对 "XiaoMing" 进行计数。

    

2.  **假设我们的邮箱现在有了更加强大的功能，容量大幅增加而且记录邮件来源，可以实现“回信”。考虑一个多核场景，有 m 个核为消费者，n 个为生产者，消费者通过邮箱向生产者提出订单，生产者通过邮箱回信给出产品。**

    -   **假设你的邮箱实现没有使用锁等机制进行保护，在多核情景下可能会发生哪些问题？单核一定不会发生问题吗？为什么？**

        多核情景下可能出现的问题：多个核同时查询某个核的邮箱容量，查到有空余并同时向该核的邮箱写入，产生冲突。

        单核情景下可能出现的问题：多个进程同时向一个进程的邮箱写可能会产生问题。具体的情景是，生产者 P1 查询消费者 C1 的邮箱是否已满，结果为未满，决定立刻向 C1 的邮箱写入；生产者 P2 查询 C1 的邮箱是否已满，结果为未满，决定立刻向 C1 的邮箱写入；P1 向 C1 的邮箱写入；P2 也向 C1 的邮箱写入，产生冲突。这是因为查询和写入之间可能被时钟中断打断，造成查询时和写入时邮箱的容量不一致。

    -   **请结合你在课堂上学到的内容，描述读者写者问题的经典解决方案，必要时提供伪代码。**

        对不可以多个进程同时访问的资源施加互斥锁；上面例子中的查询与写入的两步应该是一个原子操作，不能被别的进程打断。可以采用信号量来解决读写冲突。
    
        初始化：
    
        ```rust
        // mailbox capacity: n
        mutex = Mutex::new();
        reading_sem = Semaphore::new(0);
        writing_wem = Semaphore::new(n);
        ```
    
        写操作的伪代码如下：
    
        ```rust
        writing_sem.sub(); // blocked if writing_sem <= 0
        mutex.acquire();
        write_mail();
        mutex.release();
        reading_sem.add();
        ```
    
        读操作的伪代码如下：
    
        ```rust
        reading_sem.sub(); // blocked if reading_sem <= 0
        mutex.acquire();
        read_mail();
        mutex.release();
        writing_sem.add();
        ```
    
    -   **由于读写是基于报文的，不是随机读写，你有什么点子来优化邮箱的实现吗？**
    
        如果每次读写不一定是一个报文，而是随机数量的报文，可以在写邮件/读邮件的系统调用中增加读写的报文数。在读写前进入临界区，查询是否有足够邮件/是否有足够容量；否则退出临界区进行忙等待；是则开始读写，完成后退出临界区。