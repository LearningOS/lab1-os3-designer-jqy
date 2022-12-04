
mod context;

use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

use self::context::TrapContext;

core::arch::global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

// 使能时钟中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
// 处理中断、异常或者来自应用的系统调用
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    // 获取 Trap 产生的原因
    let scause = scause::read();
    // 获取 Trap 的附加信息
    let stval = stval::read();
    match scause.cause() {
        // 由 ecall 指令触发的系统调用，在进入 Trap 的时候
        // 硬件会将 sepc 设置为这条 ecall 指令所在的地址（因为它是进入 Trap 之前最后一条执行的指令）
        Trap::Exception(Exception::UserEnvCall) => {
            // 应用系统调用触发 Trap 的情况, 设置系统返回的地址为触发指令的下一条指令
            // (在 Trap 返回之后，我们希望应用程序控制流从 ecall 的下一条指令 开始执行。
            // 因此我们只需修改 Trap 上下文里面的 sepc，让它增加 ecall 指令的码长，也即 4 字节。)
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!("[kernel] PageFault in applicationm, bad addr = {:#x}, bad instruction = {:#x}, core dumped.", stvec, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::UserTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsuported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}