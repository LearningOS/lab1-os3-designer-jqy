.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # 当前 sp 替换为内核栈指针, sscratch 替换为用户栈指针
    # 分配 34x8 字节的栈空间(TrapContext的大小)
    addi sp, sp, -34*8
    # 保存一般寄存器
    sd x1, 1*8(sp)
    # 跳过 x2, 我们后面保存 
    sd x3, 3*8(sp)
    # 跳过 x4, 应用不使用
    # 保存 x5~x31 寄存器
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 可用自由使用 t0/t1/t2 寄存器, 因为它们已经在内核栈被保存
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 从 sscratch 读取用户栈地址, 并保存在内核栈上
    csrr t2, sscratch
    sd t2, 2*8(sp)
    mv a0, sp
    call trap_handler

__restore:
    # 当前 sp 指向内核栈(在分配队内存后), sscratch 指向用户栈
    # 恢复 sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # 恢复一般寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 释放掉 34x8 字节的栈空间(TrapContext的大小)
    add sp, sp, 34*8
    csrrw sp, sscratch, sp
    sret
