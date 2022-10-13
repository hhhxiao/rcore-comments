### OS1

> 这边主要说一些我个人理解以及原版文档没说然后我自己去查资料的地方。

### 概述

ch1部分严格来说不算是一个操作系统，而是一个能运行在裸机上的程序(更严格来说是一段指令序列)，编译之前的各种预处理，链接脚本等等工作都是为了能让`os.bin`能顺利运行在QEMU模拟的硬件环境上。

### 为什么OS需要一个栈

`entry.asm`上定义了一个栈,而这个栈是运行程序所必须的：**函数的调用需要进行参数的传递，结果的暂存和传递**这一过程需要栈的参与。而CPU从始至终都有且只有一个栈指针sp(即x2),因此当你**使用不同的运行栈的时候需要及时切换栈指针**

```assembly
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
```

上面这行代码就在`bss`段中定义了一个栈，用于函数的执行（这里暂时不考虑爆栈等问题，先跑起来再说）。

### 链接脚本是干啥用的

#### QEMU的物理内存布局

为了使QEMU能正常加载OS镜像,这里需要首先明确QEMU的物理内存布局

```
[						  ]
[ OS内核镜像加载到内存后的布局] 0x80200000
[						  ]
[   BootLoader 起始位置    ]0x80000000 
[						  ]
[ 第一条指令(jmp)           ] 0x1000
```

#### 内核镜像

编译好的内核镜像`os.bin`就是一个普通的elf文件，包括一个header和多个section，一个可能的elf文件结构为：

```
[header   ]
[section 1]
[section 2]
[   ...   ]
[section n]
```

ELF内部还定义了其被载入到内存后每个段的内容布局，这个布局是ELF文件生成过程中由连接器决定的，一个常见的内存布局是这样的：

```
[		  ]
[  data   ]
[  text   ]
```

链接器有默认的内存布局设定，但是在当前的情境(QEMU)下，连接器的默认布局不适合我们(即QEMU)使用(见下一节)

为了自定义内存布局,这里使用ld支持的链接脚本:

```c++
OUTPUT_ARCH(riscv)  //定义输出架构
ENTRY(_start)     //定义执行的入口地址
BASE_ADDRESS = 0x80200000; 
//下面就是详细的内存布局了
SECTIONS
{
    . = BASE_ADDRESS;//告诉连接器将载入到内存的起始地址定义为BASE_ADDRESS的值,也就是0x80200000
    skernel = .; //定义一个外部变量

    stext = .; //定义一个外部变量
    .text : { //定义text段在内存的位置
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
	//下面的类似
}
```

这一段脚本的主要做作用有两个:

1. 定义os镜像加载到内存的基地址为0x80200000
2. 定义一些外部变量如`rodata,srodata`等方便debug

