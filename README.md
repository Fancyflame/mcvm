# MCVM

MCVM是一门能够编译为minecraft指令的编程语言，其利用计分板作为内存。
MCVM只保证在Bedrock版本运行。

## 编译

目前没有提供编译后可执行程序，您需要先[安装Rust](https://www.rust-lang.org/zh-CN/tools/install)来编译MCVM。

```
cargo r path/to/behavior_pack
```

该指令会在当前文件夹下新建`functions`文件夹，并在内生成mcfunction文件。您需要自己新建`manifest.json`以在游戏内访问。

## 编译环境变量

- `MCVM_MEM_SIZE`：分配内存大小（计分板项数量），必须是2的n次幂，可以为0，默认为128。

## 直接访问内存

您可以操作“指针”和“寄存器”来往内存中读写值。

您需要先初始化内存
```
/function init
```

您可以通过下面的指令将数据114514写入内存的第105位（0是第一位）

```
/scoreboard players set MCVM_Memory MCVM_Memory_Reg 114514
/scoreboard players set MCVM_Memory MCVM_Memory_Ptr 104
/function store
```

然后，您可以通过下面的指令将数据从内存的第105位读取到寄存器

```
/scoreboard players set MCVM_Memory MCVM_Memory_Ptr 104
/function load
```

您也可以交换内存中的值和寄存器中的值

```
/scoreboard players set MCVM_Memory MCVM_Memory_Ptr 104
/function swap
```
