# lab3 实验报告

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
1. 迁移之前的 get_time 与 mmap 等系统调用
2. 实现了 spawn 系统调用。比较简单，可以看作是 fork + exec 的组合。只是省略了先 copy 父进程的内存空间的步骤。
3. 实现了 stride 调度算法。主要是修改了 task_manager 的 fetch 方法，找出当前 stride 最小的进程进行调度；run_tasks 调用时也增加其 stride 即可。

## 问答作业
stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

1. 实际情况是轮到 p1 执行吗？为什么？
> 实际情况不会轮到p1执行。原因如下：
> 1. 使用8位无符号整数存储时，p2执行后stride变为250 + 10 = 260
> 2. 260会溢出变为260 - 256 = 4
> 3. 比较时p1.stride=255 > p2.stride=4，系统会错误地选择p2继续执行
> 
> 这是因为无符号数溢出后数值"回绕"，导致调度器无法正确识别哪个进程的stride值实际上更小。

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

2. 为什么？尝试简单说明（不要求严格证明）。
> 当所有进程优先级≥2时：
> 1. 每个进程的步长增量 stride_pass = BigStride / priority ≤ BigStride/2
> 2. 设当前最小stride为S_min，当某进程被选中执行后，其新stride = S_min + stride_pass
> 3. 由于stride_pass ≤ BigStride/2，则新stride - S_min ≤ BigStride/2
> 4. 其他进程的stride在此期间保持不变，因此最大差值不会超过BigStride/2
> 
> 这样设计保证了即使有进程刚更新过stride，它与当前最小stride的差值也不会超过阈值，从而避免溢出导致的比较错误。

已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const BIG_STRIDE: u64 = 255;
        let diff = self.0.wrapping_sub(other.0);
        if diff <= BIG_STRIDE / 2 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: (125 < 255) == false, (129 < 255) == true.