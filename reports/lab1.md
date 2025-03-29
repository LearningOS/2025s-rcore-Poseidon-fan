# lab1 实验报告

> 1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
> 
> 无
>
> 2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
>
> 无
>
> 3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
> 
> 4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

## 实现的功能
主要多实现了一个`sys_get_time`系统调用，根据不同的输入参数执行读/写内存或统计系统调用次数的功能。
1. 读写内存：这部分比较简单，直接在`unsafe`块中将引用强转成裸指针后读取内存/解引用即可；
2. 统计调用次数：计数功能的实现：由于已经实现了 `global_allocator`，所以可以使用内置的 `BTreeMap` 来实现计数功能。具体地，在`TaskManagerInner`中添加一个`BTreeMap`的数组，数组下标代表第几个 task，键为 syscall_id，值为调用次数。
有一点注意的是`BTreeMap`没有实现`Copy` trait，所以需要先初始化一个 const 变量，然后在 manager 中进行赋值。在每次进入系统调用前，先对当前的 task 中的相应系统调用次数加一即可。

## 简答作业

### 1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

会报错：
```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Panicked at src/syscall/fs.rs:11 called `Result::unwrap()` on an `Err` value: Utf8Error { valid_up_to: 3, error_len: Some(1) }
```
首先 trap 到内核的 trapHanddler 中，分发到非法指令分支处理，打印出错信息后杀死该任务，进而调度下一个任务。

### 2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
#### 2.1 L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
刚进入 `__restore` 时，sp 代表了内核栈顶的地址。
第一种场景是内核处理好 trap 后，要恢复到用户态运行，通过 `__restore` 实现（回复上下文后执行 `sret`）

#### 2.2 L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
```
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
主要用于恢复应用程序上下文：
- `t0`：保存的 `sstatus` 寄存器的值，用于恢复 `sstatus` 寄存器
- `t1`：保存的 `sepc` 寄存器的值，用于恢复 `sepc` 寄存器
- `t2`：保存的 `sscratch` 寄存器的值，用于恢复 `sscratch` 寄存器
分别代表了 trap 前的特权寄存器，异常返回地址，和用户栈。

#### 2.3 L50-L56：为何跳过了 x2 和 x4？
```
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```
x0 被硬编码为 0，不会改变，x4寄存器的值一般也不会变化。
x2 是栈指针，要在最后进行换栈恢复。

#### 2.4 L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```
csrrw sp, sscratch, sp
```
把 sp 换回到用户栈，sscratch 指向内核栈。

#### 2.5 __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
在 sret 指令执行之后会进入用户态。这是riscv架构的规定。

#### 2.6 L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```
csrrw sp, sscratch, sp
```
把 sp 换回到内核栈，sscratch 指向用户栈。

#### 2.7 从 U 态进入 S 态是哪一条指令发生的？
ecall 指令。

