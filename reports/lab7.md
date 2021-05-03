# lab7

计86 罗境佳 2018013469



## 实验报告

### 实验指导

本次实验在上一章的基础上增加了一个简易文件系统，主要在 `easy-fs` 中，它由上到下被分为 5 层：磁盘块管理层，块缓存全局管理器，磁盘数据结构层，块缓存层，磁盘块设备接口层。一个 `Block` 是读写与换入换出的基本单位，而一个 `Block` 内部包含多个 `Inode`；从系统角度看，每个 `Inode` 被抽象为一个文件，`inode_id` 与一组 `(block_id, block_offset)` 一一对应。

在内核中，一个 `Inode` 被包装为实现了 `File trait` 的 `OSInode`。文件系统的根节点 `ROOT_INODE` 是一个目录，其数据由多个 `DirEntry` 组成，里面存储了每个子结点的名字和 `inode_id`，而 `EasyFileSystem` 可以通过 `inode_id` 则可以定位到具体的文件。简单起见，本节实验仅实现了单层的文件树。

### 编程作业

编程作业主要完成了 3 个系统调用。

-   `linkat`：首先根据 `old_path` 定位到该文件的 `inode_id`，给 `ROOT_INODE` 增加一个 `DirEntry`，里面存储 `new_path` 以及旧文件的 `inode_id`，实际上并不开辟新的磁盘空间给 `new_path`。
-   `unlinkat`：找到 `ROOT_INODE` 底下对应的 `DirEntry`，将其清空。
-   `fstat`：需要统计文件的 device, inode, mode, nlink 等数据。其中 mode 是通过对应的 `DickInode` 的类型来得到，nlink 是计算 `ROOT_INODE` 底下的 `DirEntry` 中与 target inode_id 一致的数量。



## 问答作业

1.  **目前的文件系统只有单级目录，假设想要支持多级文件目录，请描述你设想的实现方式，描述合理即可。**

    -   目录可以视作一种特殊的文件，其储存的数据是若干个 `DirEntry`。对于多层文件目录中的 `Dir` 类型，其内部存储的 `DirEntry` 可能包括文件和目录，但是并没有什么本质不同。

    -   需要增加文件路径-文件的映射方式，对于一个 `**/**/**` 形式的路径表示，其实现是先按 `/` 分开，再逐层调用目录节点的 `find` 方法。
    -   可以在 `Inode` 中存储父节点的 `inode_id` 等等，以支持 `cd ..` 的操作。

2.  **在有了多级目录之后，我们就也可以为一个目录增加硬链接了。在这种情况下，文件树中是否可能出现环路（软硬链接都可以，鼓励多尝试）？你认为应该如何解决？请在你喜欢的系统上实现一个环路，描述你的实现方式以及系统提示、实际测试结果。**

    尝试创建硬链接：

    ```shell
    gloriallluo@Luode-MacBook-Pro ~ % ln /Users/gloriallluo /gloriallluo
    ln: /Users/gloriallluo: Is a directory
    ```

    通过软链接创建环路：

    ```shell
    gloriallluo@Luode-MacBook-Pro child % pwd
    /Users/gloriallluo/parent/child
    gloriallluo@Luode-MacBook-Pro child % ln -s .. ./back
    gloriallluo@Luode-MacBook-Pro child % cd back
    gloriallluo@Luode-MacBook-Pro back % pwd
    /Users/gloriallluo/parent/child/back
    gloriallluo@Luode-MacBook-Pro back % ls
    child
    gloriallluo@Luode-MacBook-Pro back % cd child
    gloriallluo@Luode-MacBook-Pro child % pwd
    /Users/gloriallluo/parent/child/back/child
    gloriallluo@Luode-MacBook-Pro child % ls
    back
    ```

    

