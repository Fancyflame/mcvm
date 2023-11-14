# MCVM

- [MCVM](#mcvm)
  - [编译](#编译)
  - [编译环境变量](#编译环境变量)
  - [直接访问内存](#直接访问内存)
  - [汇编指令](#汇编指令)

MCVM是一门能够编译为minecraft指令的编程语言，其利用计分板作为内存。
MCVM只保证在Bedrock版本运行。

## 编译

目前没有提供编译后可执行程序，您需要先[安装Rust](https://www.rust-lang.org/zh-CN/tools/install)来编译MCVM。

```
cargo r path/to/input/file path/to/output/behavior_pack/root/folder
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
/scoreboard players set MCVM_Memory MCVM_Memory_RegR0 114514
/scoreboard players set MCVM_Memory MCVM_Memory_Pointer 104
/function MCVM_Memory_Store
```

然后，您可以通过下面的指令将数据从内存的第58位读取到寄存器R0

```
/scoreboard players set MCVM_Memory MCVM_Memory_Pointer 57
/function MCVM_Memory_Load
```

您也可以交换内存中第111位的值和寄存器R0中的值

```
/scoreboard players set MCVM_Memory MCVM_Memory_Pointer 110
/function MCVM_Memory_Swap
```

## 汇编指令

用于本项目编译到minecraft指令的汇编语言叫Mas（mcvm assembly），下面是指令集语法表，
源文件在`mas.txt`文件中，[教程链接在此](InstructionGuide.md)。
```
<label>         ::= <ident>:
<addr>          ::= <int>
<cmp-op>        ::= "<" | ">" | "<=" | ">=" | =
<calc-op>       ::= + | - | * | / | % | < | >
<range>         ::= <|lb:int>..<|hb:int> | <int>
<reg>           ::= R0|R1|R2|R3

<raw-command>   ::= cmd <string>
<move>          ::= mov <dst:reg> <src:reg>
<set>           ::= set <reg> <int>
<load>          ::= load <addr>
<store>         ::= store <addr>
<cmp>           ::= cmp <cmp-op>
<cmp-in>        ::= cmpin <range>
<branch>        ::= b <label:ident>
<branch-if>     ::= bi <label:ident>
<branch-if-not> ::= bn <label:ident>
<calculate>     ::= calc <calc-op>
<random>        ::= rand <min:int> <max:int>
<call>          ::= call <int> <label>
<debug>         ::= debug <string>
<log>           ::= log <string>
```
